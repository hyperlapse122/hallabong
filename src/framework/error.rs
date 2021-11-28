use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Invalid Arguments")]
    InvalidArguments,
    #[error("Invalid Arguments: {0}")]
    DetailedInvalidArguments(String),

    #[error("Already deafened")]
    AlreadyDeafened,
    #[error("Already muted")]
    AlreadyMuted,
    #[error("Not in voice channel")]
    NotInVoiceChannel,
    #[error("Not seekable")]
    NotSeekable,
    #[error("Songbird Voice client placed in at initialization")]
    SongbirdInitialization,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("Unknown Error")]
    Unknown,
}
