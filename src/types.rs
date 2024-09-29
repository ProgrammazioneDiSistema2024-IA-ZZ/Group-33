#[derive(Debug, PartialEq)]
pub enum BackupState {
    Idle,            // The program is waiting for user input
    Confirming,      // The user is confirming the backup action
    Confirmed,       // The backup has been confirmed by the user
    BackingUp,       // The backup process is in progress
}
