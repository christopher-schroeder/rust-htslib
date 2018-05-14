// Copyright 2014 Christopher Schröder, Johannes Köster.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Module for working with SAM files.

use std::ffi;
use std::path::Path;

use htslib;

use bam::HeaderView;
use bam::header;
use bam::record;

/// SAM writer.
#[derive(Debug)]
pub struct Writer {
    f: *mut htslib::htsFile,
    header: HeaderView,
}

/// Wrapper for opening a SAM file.
fn hts_open(path: &ffi::CStr, mode: &[u8]) -> Result<*mut htslib::htsFile, WriterError> {
    let ret = unsafe { htslib::hts_open(path.as_ptr(), ffi::CString::new(mode).unwrap().as_ptr()) };
    if ret.is_null() {
        Err(WriterError::IOError)
    } else {
        Ok(ret)
    }
}

impl Writer {
    /// Create new SAM file writer.
    ///
    /// # Arguments
    ///
    /// * `path` - the path.
    /// * `header` - header definition to use
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        header: &header::Header,
    ) -> Result<Self, WriterError> {
        if let Some(p) = path.as_ref().to_str() {
            Ok(try!(Self::new(p.as_bytes(), header)))
        } else {
            Err(WriterError::IOError)
        }
    }

    /// Create a new SAM file at STDOUT.
    ///
    /// # Arguments
    ///
    /// * `header` - header definition to use
    pub fn from_stdout(header: &header::Header) -> Result<Self, WriterError> {
        Self::new(b"-", header)
    }

    fn new(path: &[u8], header: &header::Header) -> Result<Self, WriterError> {
        let f = try!(hts_open(&ffi::CString::new(path).unwrap(), b"w"));
        let header_view = HeaderView::from_header(header);

        unsafe {
            htslib::sam_hdr_write(f, &header_view.inner());
        }
        Ok(Writer {
            f: f,
            header: header_view,
        })
    }

    /// Write record to SAM.
    ///
    /// # Arguments
    ///
    /// * `record` - the record to write
    pub fn write(&mut self, record: &record::Record) -> Result<(), WriteError> {
        if unsafe { htslib::sam_write1(self.f, &self.header.inner(), record.inner) } == -1 {
            Err(WriteError::Some)
        } else {
            Ok(())
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        unsafe {
            htslib::hts_close(self.f);
        }
    }
}

quick_error! {
    #[derive(Debug, Clone)]
    pub enum WriterError {
        IOError {}
    }
}

quick_error! {
    #[derive(Debug, Clone)]
    pub enum WriteError {
        Some {
            description("error writing record")
        }
    }
}

#[cfg(test)]
mod tests {
    use bam::Read;
    use bam::Reader;
    use bam::header;
    use bam::record;
    use sam::Writer;

    #[test]
    fn test_sam_writer_example() {
        fn from_bam_with_filter<'a, 'b, F>(bamfile: &'a str, samfile: &'b str, f: F) -> bool
        where
            F: Fn(&record::Record) -> Option<bool>,
        {
            let mut bam_reader = Reader::from_path(bamfile).unwrap(); // internal functions, just unwarp
            let header = header::Header::from_template(bam_reader.header());
            let mut sam_writer = Writer::from_path(samfile, &header).unwrap();
            for record in bam_reader.records() {
                if record.is_err() {
                    return false;
                }
                let parsed = record.unwrap();
                match f(&parsed) {
                    None => return true,
                    Some(false) => {}
                    Some(true) => if let Err(_) = sam_writer.write(&parsed) {
                        return false;
                    },
                }
            }
            true
        }
        use std::fs::File;
        use std::io::Read;
        let bamfile = "./test/bam2sam_test.bam";
        let samfile = "./test/bam2sam_out.sam";
        let expectedfile = "./test/bam2sam_expected.sam";
        let result = from_bam_with_filter(bamfile, samfile, |_| Some(true));
        assert!(result);
        let mut expected = Vec::new();
        let mut written = Vec::new();
        assert!(
            File::open(expectedfile)
                .unwrap()
                .read_to_end(&mut expected)
                .is_ok()
        );
        assert!(
            File::open(samfile)
                .unwrap()
                .read_to_end(&mut written)
                .is_ok()
        );
        assert_eq!(expected, written);
    }
}
