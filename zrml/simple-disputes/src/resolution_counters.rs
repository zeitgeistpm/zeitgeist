#[derive(Debug, Default)]
pub struct ResolutionCounters {
    pub total_accounts: u32,
    pub total_asset_accounts: u32,
    pub total_categories: u32,
    pub total_disputes: u32,
}

impl ResolutionCounters {
    #[inline]
    pub(crate) fn saturating_add(&mut self, other: &Self) {
        self.total_accounts = self.total_accounts.saturating_add(other.total_accounts);
        self.total_asset_accounts +=
            self.total_asset_accounts.saturating_add(other.total_asset_accounts);
        self.total_categories += self.total_categories.saturating_add(other.total_categories);
        self.total_disputes += self.total_disputes.saturating_add(other.total_disputes);
    }
}
