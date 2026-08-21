use mingling::macros::entry;
use mingling::{ChainInvoker, macros::command};

pub(crate) mod docsify_refresh;
pub(crate) mod example_refresh;
pub(crate) mod features_refresh;

use crate::tools::docsify_refresh::EntryDocsifyRefresh;
use crate::tools::example_refresh::EntryExampleRefresh;
use crate::tools::features_refresh::EntryFeaturesRefresh;

#[command]
pub async fn refresh(
    docsify: &ChainInvoker<EntryDocsifyRefresh>,
    example: &ChainInvoker<EntryExampleRefresh>,
    features: &ChainInvoker<EntryFeaturesRefresh>,
) {
    docsify.invoke_to_last(entry!()).await;
    example.invoke_to_last(entry!()).await;
    features.invoke_to_last(entry!()).await;
}
