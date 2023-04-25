use frame_support::dispatch::DispatchError;

pub trait VoteValidation {
    fn pre_validate() -> Result<(), DispatchError>;
}