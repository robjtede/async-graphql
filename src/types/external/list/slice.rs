use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{registry, ContextSelectionSet, OutputValueType, Positioned, Result, Type};
use std::borrow::Cow;

impl<'a, T: Type + 'a> Type for &'a [T] {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for &[T] {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        resolve_list(ctx, field, self.iter()).await
    }
}
