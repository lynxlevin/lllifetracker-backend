use chrono::{DateTime, FixedOffset};
use db_adapters::{
    action_track_adapter::{
        ActionTrackAdapter, ActionTrackFilter, ActionTrackLimit, ActionTrackOrder, ActionTrackQuery,
    },
    user_adapter::{UserAdapter, UserMutation},
    Order::Asc,
};
use entities::{action_track, user};

use crate::UseCaseError;

enum UseCase {
    CreateTrack(CreateTrack),
    UpdateTrack(UpdateTrack),
    DeleteTrack(DeleteTrack),
    SwitchActionArchive,
}

enum CreateTrack {
    FirstTrack,
    OlderTrack,
    NewerTrack,
}

enum UpdateTrack {
    FirstTrackToOlder,
    FirstTrackToNewer,
    NewerTrackToOlder,
    NewerTrackToNewer,
}

enum DeleteTrack {
    FirstTrack,
    NewerTrack,
}

pub struct FirstTrackAtSynchronizer<'a> {
    action_track_adapter: ActionTrackAdapter<'a>,
    user_adapter: UserAdapter<'a>,
    user: user::Model,
}

impl<'a> FirstTrackAtSynchronizer<'a> {
    pub fn init(
        action_track_adapter: ActionTrackAdapter<'a>,
        user_adapter: UserAdapter<'a>,
        user: user::Model,
    ) -> Self {
        Self {
            action_track_adapter,
            user_adapter,
            user,
        }
    }

    pub async fn update_user_first_track_at(
        self,
        old_track: Option<action_track::Model>,
        new_track: Option<action_track::Model>,
    ) -> Result<(), UseCaseError> {
        let update_type = match (&old_track, &new_track) {
            (None, Some(new_track)) => UseCase::CreateTrack(match self.user.first_track_at {
                Some(first_track_at) => match new_track.started_at < first_track_at {
                    true => CreateTrack::OlderTrack,
                    false => CreateTrack::NewerTrack,
                },
                None => CreateTrack::FirstTrack,
            }),
            (Some(old_track), Some(new_track)) => {
                UseCase::UpdateTrack(match self.user.first_track_at {
                    Some(first_track_at) => match old_track.started_at == first_track_at {
                        true => match new_track.started_at < first_track_at {
                            true => UpdateTrack::FirstTrackToOlder,
                            false => UpdateTrack::FirstTrackToNewer,
                        },
                        false => match new_track.started_at < first_track_at {
                            true => UpdateTrack::NewerTrackToOlder,
                            false => UpdateTrack::NewerTrackToNewer,
                        },
                    },
                    None => UpdateTrack::NewerTrackToOlder,
                })
            }
            (Some(old_track), None) => UseCase::DeleteTrack(match self.user.first_track_at {
                Some(first_track_at) => match old_track.started_at == first_track_at {
                    true => DeleteTrack::FirstTrack,
                    false => DeleteTrack::NewerTrack,
                },
                None => DeleteTrack::NewerTrack,
            }),
            (None, None) => UseCase::SwitchActionArchive,
        };

        match update_type {
            UseCase::CreateTrack(CreateTrack::FirstTrack)
            | UseCase::CreateTrack(CreateTrack::OlderTrack)
            | UseCase::UpdateTrack(UpdateTrack::FirstTrackToOlder)
            | UseCase::UpdateTrack(UpdateTrack::NewerTrackToOlder) => {
                self._update_first_track_at(Some(new_track.unwrap().started_at))
                    .await
            }
            UseCase::UpdateTrack(UpdateTrack::FirstTrackToNewer)
            | UseCase::DeleteTrack(DeleteTrack::FirstTrack)
            | UseCase::SwitchActionArchive => {
                self._search_for_first_action_track_and_update_user().await
            }
            UseCase::CreateTrack(CreateTrack::NewerTrack)
            | UseCase::UpdateTrack(UpdateTrack::NewerTrackToNewer)
            | UseCase::DeleteTrack(DeleteTrack::NewerTrack) => Ok(()),
        }
    }

    async fn _search_for_first_action_track_and_update_user(self) -> Result<(), UseCaseError> {
        let action_tracks = self
            .action_track_adapter
            .clone()
            .filter_eq_user(&self.user)
            .filter_eq_archived_action(false)
            .order_by_started_at(Asc)
            .limit(1)
            .get_all()
            .await
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

        self._update_first_track_at(
            action_tracks
                .first()
                .and_then(|track| Some(track.started_at)),
        )
        .await
    }

    async fn _update_first_track_at(
        self,
        first_track_at: Option<DateTime<FixedOffset>>,
    ) -> Result<(), UseCaseError> {
        self.user_adapter
            .update_first_track_at(self.user, first_track_at)
            .await
            .map(|_| ())
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
    }
}
