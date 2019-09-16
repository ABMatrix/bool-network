// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use proptest::collection::vec;

fn hash_blob(blob: &[u8]) -> HashValue {
    let mut hasher = AccountStateBlobHasher::default();
    hasher.write(blob);
    hasher.finish()
}