#![cfg_attr(feature = "better-docs",
    cfg_attr(all(), doc = include_str!("../README.md"))
)]
#![cfg_attr(feature = "better-docs",
    feature(doc_auto_cfg),
)]


pub mod panic;
pub mod send;
