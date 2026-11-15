use commonware_consensus::{
    simplex::elector::{self, Dynamic, Terms},
    types::Round,
};
use commonware_cryptography::certificate::Scheme;
use commonware_utils::{Participant, ordered::Set};

#[derive(Clone, Debug)]
pub(crate) struct MemberElector(pub [u32; 5]);

impl<S: Scheme> elector::Config<S> for MemberElector {
    type Elector = Self;
    fn build(self, participants: &Set<S::PublicKey>) -> Self {
        assert_eq!(participants.len(), 5);
        self
    }
}
impl<S: Scheme> elector::Elector<S> for MemberElector {
    type Mode = Dynamic;
    fn terms(&self) -> Terms {
        Terms::rotating()
    }
    fn elect(&self, round: Round, _: Option<&S::Certificate>) -> Participant {
        let member = (round.view().get().checked_sub(1).expect("view 1 is first") % 5) as usize;
        Participant::new(self.0[member])
    }
}
