pub struct Probabilistic<TEvent> {
    pub event: TEvent,
    pub probability: f64,
}

impl<TEvent> Probabilistic<TEvent> {
    pub fn new(event: TEvent, probability: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&probability),
            "The probability {probability} is not valid."
        );
        Self { event, probability }
    }

    pub fn deterministic(event: TEvent) -> Self {
        Self {
            event,
            probability: 1.0,
        }
    }

    pub fn many_uniform(
        events: impl IntoIterator<IntoIter: ExactSizeIterator, Item = TEvent>,
    ) -> Vec<Probabilistic<TEvent>> {
        let iter = events.into_iter();
        let probability = 1.0 / iter.len() as f64;
        iter.map(|event| Probabilistic::new(event, probability))
            .collect()
    }

    pub fn many_from_mapping(
        mapping: impl IntoIterator<Item = (TEvent, f64)>,
    ) -> Vec<Probabilistic<TEvent>> {
        mapping
            .into_iter()
            .map(|(event, prob)| Probabilistic::new(event, prob))
            .collect()
    }
}
