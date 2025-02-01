// This file is part of the uutils sed package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.
#[macro_use]
mod common;

#[cfg(feature = "sed")]
#[path = "by-util/test_sed.rs"]
mod test_sed;
