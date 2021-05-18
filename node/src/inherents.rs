use sp_core::crypto::AccountId32;
use sp_inherents::InherentDataProviders;

/// Build the inherent data providers for the node.
///
/// Not all nodes will need all inherent data providers:
/// - The author provider is only necessary for block producing nodes
/// - The validation data provider can be mocked.
pub fn build_inherent_data_providers(
    author_id: Option<AccountId32>,
) -> Result<InherentDataProviders, sc_service::Error> {
    let providers = InherentDataProviders::new();

    // Timestamp provider. Needed in all nodes.
    providers
        .register_provider(sp_timestamp::InherentDataProvider)
        .map_err(Into::into)
        .map_err(sp_consensus::error::Error::InherentData)?;

    // Author ID Provider for authoring node only.
    if let Some(inner) = author_id {
        providers
            .register_provider(pallet_author_inherent::InherentDataProvider(inner))
            .map_err(Into::into)
            .map_err(sp_consensus::error::Error::InherentData)?;
    }

    Ok(providers)
}
