//! Pure ownership model for the fixed XR819 host-TX context arena.
//!
//! Hardware context addresses define slot identity. The model deliberately
//! separates revocability from pipeline stage so cancellation and release
//! checks do not rely on fragile lists of phase variants.

pub const HOST_CONTEXT_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextId(u8);

impl ContextId {
    pub const fn new(index: u8) -> Option<Self> {
        if (index as usize) < HOST_CONTEXT_COUNT {
            Some(Self(index))
        } else {
            None
        }
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HardwareOwnership {
    Reversible,
    HardwareOwned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Free,
    Retained,
    Pending,
    PasQueued,
    Reserved,
    Scheduled,
    ConfirmationReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SlotState {
    pub ownership: HardwareOwnership,
    pub stage: Stage,
    pub packet_id: u32,
    pub interface: u8,
    pub epoch: u8,
}

impl SlotState {
    pub const FREE: Self = Self {
        ownership: HardwareOwnership::Reversible,
        stage: Stage::Free,
        packet_id: 0,
        interface: 0,
        epoch: 0,
    };

    pub const fn occupied(self) -> bool {
        !matches!(self.stage, Stage::Free)
    }

    pub const fn cancellable(self) -> bool {
        matches!(self.ownership, HardwareOwnership::Reversible)
            && matches!(
                self.stage,
                Stage::Retained | Stage::Pending | Stage::PasQueued | Stage::Reserved
            )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionError {
    InvalidTransition,
    HardwareOwned,
    SlotOccupied,
    SlotFree,
    DuplicatePacketId,
}

pub const fn transition_allowed(from: SlotState, to: Stage) -> bool {
    matches!(
        (from.stage, to),
        (Stage::Retained, Stage::Pending)
            | (Stage::Pending, Stage::PasQueued)
            | (Stage::Pending, Stage::ConfirmationReady)
            | (Stage::PasQueued, Stage::Reserved)
            | (Stage::PasQueued, Stage::ConfirmationReady)
            | (Stage::Reserved, Stage::PasQueued)
            | (Stage::Reserved, Stage::Scheduled)
            | (Stage::Scheduled, Stage::ConfirmationReady)
    )
}

pub struct ArenaModel<const N: usize> {
    slots: [SlotState; N],
}

impl<const N: usize> ArenaModel<N> {
    pub const fn new() -> Self {
        Self {
            slots: [SlotState::FREE; N],
        }
    }

    pub fn slot(&self, id: ContextId) -> Option<SlotState> {
        self.slots.get(id.index()).copied()
    }

    pub fn occupy(
        &mut self,
        id: ContextId,
        packet_id: u32,
        interface: u8,
        epoch: u8,
    ) -> Result<(), TransitionError> {
        if self
            .slots
            .iter()
            .any(|slot| slot.occupied() && slot.packet_id == packet_id)
        {
            return Err(TransitionError::DuplicatePacketId);
        }
        let slot = self
            .slots
            .get_mut(id.index())
            .ok_or(TransitionError::SlotFree)?;
        if slot.occupied() {
            return Err(TransitionError::SlotOccupied);
        }
        *slot = SlotState {
            ownership: HardwareOwnership::Reversible,
            stage: Stage::Retained,
            packet_id,
            interface,
            epoch,
        };
        Ok(())
    }

    pub fn transition(&mut self, id: ContextId, to: Stage) -> Result<(), TransitionError> {
        let slot = self
            .slots
            .get_mut(id.index())
            .ok_or(TransitionError::SlotFree)?;
        if !slot.occupied() {
            return Err(TransitionError::SlotFree);
        }
        if !transition_allowed(*slot, to) {
            return Err(TransitionError::InvalidTransition);
        }
        if matches!(to, Stage::Scheduled) {
            slot.ownership = HardwareOwnership::HardwareOwned;
        }
        slot.stage = to;
        Ok(())
    }

    pub fn cancel(&mut self, id: ContextId) -> Result<SlotState, TransitionError> {
        let slot = self
            .slots
            .get_mut(id.index())
            .ok_or(TransitionError::SlotFree)?;
        if !slot.occupied() {
            return Err(TransitionError::SlotFree);
        }
        if !slot.cancellable() {
            return Err(TransitionError::HardwareOwned);
        }
        let previous = *slot;
        *slot = SlotState::FREE;
        Ok(previous)
    }

    pub fn release_confirmation(&mut self, id: ContextId) -> Result<SlotState, TransitionError> {
        let slot = self
            .slots
            .get_mut(id.index())
            .ok_or(TransitionError::SlotFree)?;
        if !matches!(slot.stage, Stage::ConfirmationReady) {
            return Err(TransitionError::InvalidTransition);
        }
        let previous = *slot;
        *slot = SlotState::FREE;
        Ok(previous)
    }

    pub fn hardware_owned_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| matches!(slot.ownership, HardwareOwnership::HardwareOwned))
            .count()
    }

    pub fn affected_count(&self, interface: u8, maximum_epoch: u8) -> usize {
        self.slots
            .iter()
            .filter(|slot| {
                slot.occupied() && slot.interface == interface && slot.epoch <= maximum_epoch
            })
            .count()
    }
}

impl<const N: usize> Default for ArenaModel<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_slot_cannot_be_cancelled_or_reused() {
        let mut arena = ArenaModel::<HOST_CONTEXT_COUNT>::new();
        let id = ContextId::new(3).unwrap();
        arena.occupy(id, 0x1234, 0, 7).unwrap();
        arena.transition(id, Stage::Pending).unwrap();
        arena.transition(id, Stage::PasQueued).unwrap();
        arena.transition(id, Stage::Reserved).unwrap();
        arena.transition(id, Stage::Scheduled).unwrap();
        assert_eq!(arena.hardware_owned_count(), 1);
        assert_eq!(arena.cancel(id), Err(TransitionError::HardwareOwned));
        assert_eq!(
            arena.occupy(id, 0x5678, 0, 7),
            Err(TransitionError::SlotOccupied)
        );
    }

    #[test]
    fn confirmation_handoff_releases_the_model_slot() {
        let mut arena = ArenaModel::<HOST_CONTEXT_COUNT>::new();
        let id = ContextId::new(4).unwrap();
        arena.occupy(id, 0x1234, 0, 1).unwrap();
        arena.transition(id, Stage::Pending).unwrap();
        arena.transition(id, Stage::ConfirmationReady).unwrap();
        assert!(arena.slot(id).unwrap().occupied());
        assert_eq!(arena.release_confirmation(id).unwrap().packet_id, 0x1234);
        assert!(!arena.slot(id).unwrap().occupied());
        assert_eq!(
            arena.release_confirmation(id),
            Err(TransitionError::InvalidTransition)
        );
    }

    #[test]
    fn packet_ids_are_unique_across_out_of_order_slots() {
        let mut arena = ArenaModel::<HOST_CONTEXT_COUNT>::new();
        let first = ContextId::new(8).unwrap();
        let second = ContextId::new(1).unwrap();
        arena.occupy(first, 0xaaaa, 0, 1).unwrap();
        arena.occupy(second, 0xbbbb, 0, 1).unwrap();
        assert_eq!(
            arena.occupy(ContextId::new(2).unwrap(), 0xaaaa, 0, 1),
            Err(TransitionError::DuplicatePacketId)
        );
        arena.transition(second, Stage::Pending).unwrap();
        arena.transition(second, Stage::ConfirmationReady).unwrap();
        arena.release_confirmation(second).unwrap();
        assert!(arena.slot(first).unwrap().occupied());
    }

    #[test]
    fn reset_barrier_counts_only_affected_epoch() {
        let mut arena = ArenaModel::<HOST_CONTEXT_COUNT>::new();
        arena.occupy(ContextId::new(0).unwrap(), 1, 0, 3).unwrap();
        arena.occupy(ContextId::new(1).unwrap(), 2, 0, 4).unwrap();
        arena.occupy(ContextId::new(2).unwrap(), 3, 1, 2).unwrap();
        assert_eq!(arena.affected_count(0, 3), 1);
        assert_eq!(arena.affected_count(0, 4), 2);
        assert_eq!(arena.affected_count(1, 3), 1);
    }
}
