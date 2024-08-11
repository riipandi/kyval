// Copyright © 2024 Aris Ripandi - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub const DEFAULT_NAMESPACE_NAME: &str = "kv_store";

mod kyval;
pub use kyval::*;

mod store;
pub use store::*;

pub mod adapter;
