crate mod cache;
crate mod item_type;
crate mod renderer;

use rustc_hir::def_id::DefId;

crate use renderer::{run_format, FormatRenderer};

use crate::clean::{self, ItemId};
use cache::Cache;

/// Specifies whether rendering directly implemented trait items or ones from a certain Deref
/// impl.
crate enum AssocItemRender<'a> {
    All,
    DerefFor { trait_: &'a clean::Path, type_: &'a clean::Type, deref_mut_: bool },
}

/// For different handling of associated items from the Deref target of a type rather than the type
/// itself.
#[derive(Copy, Clone, PartialEq)]
crate enum RenderMode {
    Normal,
    ForDeref { mut_: bool },
}

/// Metadata about implementations for a type or trait.
#[derive(Clone, Debug)]
crate struct Impl {
    crate impl_item: clean::Item,
}

impl Impl {
    crate fn inner_impl(&self) -> &clean::Impl {
        match *self.impl_item.kind {
            clean::ImplItem(ref impl_) => impl_,
            _ => panic!("non-impl item found in impl"),
        }
    }

    crate fn trait_did(&self) -> Option<DefId> {
        self.inner_impl().trait_.as_ref().map(|t| t.def_id())
    }

    /// This function is used to extract a `DefId` to be used as a key for the `Cache::impls` field.
    ///
    /// It allows to prevent having duplicated implementations showing up (the biggest issue was
    /// with blanket impls).
    ///
    /// It panics if `self` is a `ItemId::Primitive`.
    crate fn def_id(&self) -> DefId {
        match self.impl_item.item_id {
            ItemId::Blanket { impl_id, .. } => impl_id,
            ItemId::Auto { trait_, .. } => trait_,
            ItemId::DefId(def_id) => def_id,
            ItemId::Primitive(_, _) => {
                panic!(
                    "Unexpected ItemId::Primitive in expect_def_id: {:?}",
                    self.impl_item.item_id
                )
            }
        }
    }

    // Returns true if this is an implementation on a "local" type, meaning:
    // the type is in the current crate, or the type and the trait are both
    // re-exported by the current crate.
    pub(crate) fn is_on_local_type(&self, cache: &Cache) -> bool {
        let for_type = &self.inner_impl().for_;
        if let Some(for_type_did) = for_type.def_id(cache) {
            // The "for" type is local if it's in the paths for the current crate.
            if cache.paths.contains_key(&for_type_did) {
                return true;
            }
            if let Some(trait_did) = self.trait_did() {
                // The "for" type and the trait are from the same crate. That could
                // be different from the current crate, for instance when both were
                // re-exported from some other crate. But they are local with respect to
                // each other.
                if for_type_did.krate == trait_did.krate {
                    return true;
                }
            }
            return false;
        };
        true
    }
}
