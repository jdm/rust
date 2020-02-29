use super::BackendTypes;
use rustc_middle::ty::{Instance, Ty};
use rustc_target::abi::call::FnAbi;

pub trait AbiBuilderMethods<'tcx>: BackendTypes {
    fn apply_attrs_callsite(&mut self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>, callsite: Self::Value, instance: Option<Instance<'tcx>>);
    fn get_param(&self, index: usize) -> Self::Value;
}
