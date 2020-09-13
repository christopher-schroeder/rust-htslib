// Copyright 2014 Christopher Schröder, Johannes Köster.
// Licensed under the MIT license (http://opensource.org/licenses/MIT)
// This file may not be copied, modified, or distributed
// except according to those terms.

//! Rust-Htslib provides a high level API to working with the common HTS file formats.
//!
//! Htslib itself is the *de facto* standard implementation for reading and writing files for
//! HTS alignments (SAM and BAM) as well as variant calls in VCF and BCF format.
//!
//! For example, let's say that we use samtools to view the header of a test file:
//!
//! ```bash
//! samtools view -H test/test.bam
//! @SQ    SN:CHROMOSOME_I    LN:15072423
//! @SQ    SN:CHROMOSOME_II    LN:15279345
//! @SQ    SN:CHROMOSOME_III    LN:13783700
//! @SQ    SN:CHROMOSOME_IV    LN:17493793
//! @SQ    SN:CHROMOSOME_V    LN:20924149
//! ```
//!
//! We can reproduce that with Rust-Htslib. Reading BAM files and printing the header
//! to the the screen is as easy as
//!
//! ```
//! use rust_htslib::{bam, bam::Read};
//!
//! fn main() {
//!     let bam = bam::Reader::from_path(&"test/test.bam").unwrap();
//!     let header = bam::Header::from_template(bam.header());
//!
//!     // print header records to the terminal, akin to samtool
//!     for (key, records) in header.to_hashmap() {
//!         for record in records {
//!             println!("@{}\tSN:{}\tLN:{}", key, record["SN"], record["LN"]);
//!         }
//!     }
//! }
//! ```
//!
//! which results in the following output, equivalent to samtools.
//!
//! ```bash
//! @SQ    SN:CHROMOSOME_I    LN:15072423
//! @SQ    SN:CHROMOSOME_II    LN:15279345
//! @SQ    SN:CHROMOSOME_III    LN:13783700
//! @SQ    SN:CHROMOSOME_IV    LN:17493793
//! @SQ    SN:CHROMOSOME_V    LN:20924149
//! ```
//!
//! We can also read directly from the BAM file and write to an output file
//!
//! ```
//! use rust_htslib::{bam, bam::Read};
//!
//! let mut bam = bam::Reader::from_path(&"test/test.bam").unwrap();
//! let header = bam::Header::from_template(bam.header());
//! let mut out = bam::Writer::from_path(&"test/out.bam", &header, bam::Format::BAM).unwrap();
//!
//! // copy reverse reads to new BAM file
//! for r in bam.records() {
//!     let record = r.unwrap();
//!     if record.is_reverse() {
//!         out.write(&record).unwrap();
//!     }
//! }
//! ```
//!
//! Pileups can be performed with
//!
//! ```
//! use rust_htslib::{bam, bam::Read};
//!
//! let mut bam = bam::Reader::from_path(&"test/test.bam").unwrap();
//!
//! // pileup over all covered sites
//! for p in bam.pileup() {
//!     let pileup = p.unwrap();
//!     println!("{}:{} depth {}", pileup.tid(), pileup.pos(), pileup.depth());
//!
//!     for alignment in pileup.alignments() {
//!         if !alignment.is_del() && !alignment.is_refskip() {
//!             println!("Base {}", alignment.record().seq()[alignment.qpos().unwrap()]);
//!         }
//!         // mark indel start
//!         match alignment.indel() {
//!             bam::pileup::Indel::Ins(len) => println!("Insertion of length {} between this and next position.", len),
//!             bam::pileup::Indel::Del(len) => println!("Deletion of length {} between this and next position.", len),
//!             bam::pileup::Indel::None => ()
//!         }
//!     }
//! }
//! ```
//!
//! In both cases, indexed BAM files can be seeked for specific regions, constraining either the record iterator or the pileups:
//!
//! ```
//! use rust_htslib::{bam, bam::Read};
//!
//! let mut bam = bam::IndexedReader::from_path(&"test/test.bam").unwrap();
//!
//! // seek to chr1:50000-100000
//! let tid = bam.header().tid(b"CHROMOSOME_I").unwrap();
//! bam.fetch(tid, 0, 20).unwrap();
//! // afterwards, read or pileup in this region
//! ```

#[macro_use]
extern crate custom_derive;
extern crate libc;
#[macro_use]
extern crate newtype_derive;

#[cfg(feature = "serde_feature")]
extern crate serde;
#[cfg(feature = "serde_feature")]
extern crate serde_bytes;

#[cfg(all(test, feature = "serde_feature"))]
extern crate bincode;

#[cfg(test)] // <-- not needed in examples + integration tests
#[macro_use]
extern crate pretty_assertions;
#[cfg(all(test, feature = "serde_feature"))]
extern crate serde_json;

pub mod bam;
pub mod bcf;
pub mod faidx;
pub mod htslib;
pub mod tbx;
pub mod tpool;
pub mod utils;
