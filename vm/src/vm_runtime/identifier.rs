// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! A bunch of helper functions to fetch the storage key for move resources and values.

use crate::def::{access::ModuleAccess, file_format::StructDefinitionIndex};
use crate::types::{AccessPath, Accesses, AccountAddress, ResourceKey, StructTag};

/// Get the StructTag for a StructDefinition defined in a published module.
pub fn resource_storage_key(module: &impl ModuleAccess, idx: StructDefinitionIndex) -> StructTag {
    let resource = module.struct_def_at(idx);
    let res_handle = module.struct_handle_at(resource.struct_handle);
    let res_module = module.module_handle_at(res_handle.module);
    let res_name = module.string_at(res_handle.name);
    let res_mod_addr = module.address_at(res_module.address);
    let res_mod_name = module.string_at(res_module.name);
    StructTag {
        module: res_mod_name.to_string(),
        address: *res_mod_addr,
        name: res_name.to_string(),
        type_params: vec![],
    }
}

/// Get the AccessPath to a resource stored under `address` with type name `tag`
pub fn create_access_path(address: &AccountAddress, tag: StructTag) -> AccessPath {
    let resource_tag = ResourceKey::new(*address, tag);
    AccessPath::resource_access_path(&resource_tag, &Accesses::empty())
}
