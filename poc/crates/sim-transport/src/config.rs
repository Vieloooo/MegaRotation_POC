use crate::{
    ChannelId, Endpoint, EndpointBinding, EndpointKind, Error, Id, Message, Tick, framed_hash,
    identity::MessageHeader,
    snapshot::{encode, schema},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ContextPolicy {
    Forbidden,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Reliability {
    BestEffort,
    Reliable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSpec {
    pub id: ChannelId,
    pub context: ContextPolicy,
    pub reliability: Reliability,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelPermit {
    pub channel: ChannelId,
    pub contexts: Vec<Id>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointRegistration {
    pub binding: EndpointBinding,
    pub send: Vec<ChannelPermit>,
    pub receive: Vec<ChannelPermit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum EndpointSelector {
    Any,
    Kind(EndpointKind),
    Exact(Endpoint),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ChannelSelector {
    Any,
    Exact(ChannelId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum OrderMode {
    Fifo,
    Unordered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum OfflineMode {
    Hold,
    Drop,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfflineInterval {
    pub start_us: Tick,
    pub end_us: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultEntry {
    pub send_ordinal: u64,
    pub drop: bool,
    pub duplicate_delay_us: Option<u64>,
    pub extra_delay_us: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkProfile {
    pub latency_us: u64,
    pub jitter_us: u64,
    pub order: OrderMode,
    pub offline: Vec<OfflineInterval>,
    pub offline_mode: OfflineMode,
    pub capacity: u32,
    pub drain_bytes_per_second: u64,
    pub faults: Vec<FaultEntry>,
}

impl Default for LinkProfile {
    fn default() -> Self {
        Self {
            latency_us: 20_000,
            jitter_us: 0,
            order: OrderMode::Fifo,
            offline: Vec::new(),
            offline_mode: OfflineMode::Hold,
            capacity: 64,
            drain_bytes_per_second: 1_000_000,
            faults: Vec::new(),
        }
    }
}

impl LinkProfile {
    pub(crate) fn offline_at(&self, tick: Tick) -> bool {
        let end = self.online_tick(tick);
        tick != end
    }

    pub(crate) fn online_tick(&self, mut tick: Tick) -> Tick {
        let first = self
            .offline
            .partition_point(|interval| interval.end_us <= tick);
        for interval in &self.offline[first..] {
            if tick < interval.start_us {
                break;
            }
            tick = interval.end_us;
        }
        tick
    }

    pub(crate) fn spacing(&self, payload_len: usize) -> Result<Tick, Error> {
        let micros =
            (payload_len.max(1) as u128 * 1_000_000).div_ceil(self.drain_bytes_per_second.into());
        Tick::try_from(micros.max(1)).map_err(|_| Error::ArithmeticOverflow)
    }

    fn normalize(&mut self, limits: &Limits) -> Result<(), Error> {
        if self.capacity == 0
            || self.capacity > limits.link_copies
            || self.drain_bytes_per_second == 0
            || self.jitter_us == u64::MAX
        {
            return Err(Error::InvalidConfig);
        }
        sort_unique(&mut self.offline, |i| i.start_us)?;
        if self.offline.iter().any(|i| i.start_us >= i.end_us)
            || self.offline.windows(2).any(|p| p[0].end_us > p[1].start_us)
        {
            return Err(Error::InvalidConfig);
        }
        sort_unique(&mut self.faults, |f| f.send_ordinal)?;
        if self
            .faults
            .iter()
            .any(|f| f.send_ordinal == 0 || (f.drop && f.duplicate_delay_us.is_some()))
        {
            return Err(Error::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkRule {
    pub source: EndpointSelector,
    pub destination: EndpointSelector,
    pub channel: ChannelSelector,
    pub profile: LinkProfile,
}

pub(crate) type RouteKey = (Endpoint, Endpoint, ChannelId);
type RuleKey = (EndpointSelector, EndpointSelector, ChannelSelector);

impl LinkRule {
    fn key(&self) -> RuleKey {
        (self.source, self.destination, self.channel)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub endpoints: u32,
    pub channels: u32,
    pub rules: u32,
    pub payload_bytes: u64,
    pub link_copies: u32,
    pub queued_copies: u32,
    pub queued_payload_bytes: u64,
    pub tracked_messages: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            endpoints: 64,
            channels: 32,
            rules: 1024,
            payload_bytes: 65_536,
            link_copies: 64,
            queued_copies: 1024,
            queued_payload_bytes: 4 * 1024 * 1024,
            tracked_messages: 8192,
        }
    }
}

impl Limits {
    fn validate(&self) -> Result<(), Error> {
        let max = Self::default();
        for (limit, maximum) in [
            (self.endpoints.into(), max.endpoints.into()),
            (self.channels.into(), max.channels.into()),
            (self.rules.into(), max.rules.into()),
            (self.payload_bytes, max.payload_bytes),
            (self.link_copies.into(), max.link_copies.into()),
            (self.queued_copies.into(), max.queued_copies.into()),
            (self.queued_payload_bytes, max.queued_payload_bytes),
            (self.tracked_messages.into(), max.tracked_messages.into()),
        ] {
            if limit == 0 || limit > maximum {
                return Err(Error::InvalidConfig);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransportConfig {
    pub schema: u32,
    pub run: Id,
    pub config: Id,
    pub seed: u64,
    pub limits: Limits,
    pub endpoints: Vec<EndpointRegistration>,
    pub channels: Vec<ChannelSpec>,
    pub rules: Vec<LinkRule>,
}

fn sort_unique<T, K: Ord>(entries: &mut [T], key: impl Fn(&T) -> K) -> Result<(), Error> {
    entries.sort_unstable_by_key(&key);
    if entries
        .windows(2)
        .any(|pair| key(&pair[0]) == key(&pair[1]))
    {
        return Err(Error::DuplicateId);
    }
    Ok(())
}

fn permit(entries: &[ChannelPermit], id: ChannelId) -> Result<&ChannelPermit, Error> {
    let index = entries
        .binary_search_by_key(&id, |entry| entry.channel)
        .map_err(|_| Error::PermissionDenied)?;
    Ok(&entries[index])
}

impl TransportConfig {
    pub fn normalize(self) -> Result<Self, Error> {
        let (config, _) = self.normalize_and_expand()?;
        Ok(config)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        encode(&self.clone().normalize()?)
    }

    pub fn hash(&self) -> Result<Id, Error> {
        Ok(framed_hash("SIMT/config/v1", &[&self.canonical_bytes()?]))
    }

    pub(crate) fn normalized_hash(&self) -> Result<Id, Error> {
        Ok(framed_hash("SIMT/config/v1", &[&encode(self)?]))
    }

    pub(crate) fn normalize_and_expand(
        mut self,
    ) -> Result<(Self, BTreeMap<RouteKey, usize>), Error> {
        schema(self.schema)?;
        self.limits.validate()?;
        if self.run == [0; 32] || self.config == [0; 32] {
            return Err(Error::InvalidConfig);
        }
        if self.endpoints.len() > self.limits.endpoints as usize
            || self.channels.len() > self.limits.channels as usize
            || self.rules.len() > self.limits.rules as usize
        {
            return Err(Error::ResourceLimit);
        }
        sort_unique(&mut self.endpoints, |e| e.binding.endpoint)?;
        sort_unique(&mut self.channels, |c| c.id)?;
        sort_unique(&mut self.rules, LinkRule::key)?;
        if self.channels.iter().any(|c| c.id == ChannelId(0)) {
            return Err(Error::InvalidConfig);
        }
        let mut contexts = 0;
        for endpoint in &mut self.endpoints {
            if endpoint.binding.incarnation == 0 {
                return Err(Error::InvalidConfig);
            }
            for entries in [&mut endpoint.send, &mut endpoint.receive] {
                if entries.len() > self.channels.len() {
                    return Err(Error::InvalidConfig);
                }
                sort_unique(entries, |p| p.channel)?;
                for permit in entries {
                    let index = self
                        .channels
                        .binary_search_by_key(&permit.channel, |c| c.id)
                        .map_err(|_| Error::UnknownChannel)?;
                    if permit.contexts.len() > 4096 - contexts {
                        return Err(Error::ResourceLimit);
                    }
                    contexts += permit.contexts.len();
                    sort_unique(&mut permit.contexts, |id| *id)?;
                    let forbidden = self.channels[index].context == ContextPolicy::Forbidden;
                    if (forbidden && !permit.contexts.is_empty())
                        || permit.contexts.contains(&[0; 32])
                    {
                        return Err(Error::InvalidConfig);
                    }
                }
            }
        }
        let mut intervals = 0;
        let mut faults = 0;
        for rule in &mut self.rules {
            for selector in [rule.source, rule.destination] {
                if let EndpointSelector::Exact(endpoint) = selector {
                    self.endpoints
                        .binary_search_by_key(&endpoint, |e| e.binding.endpoint)
                        .map_err(|_| Error::UnknownEndpoint)?;
                }
            }
            if let ChannelSelector::Exact(channel) = rule.channel {
                self.channels
                    .binary_search_by_key(&channel, |c| c.id)
                    .map_err(|_| Error::UnknownChannel)?;
            }
            if rule.profile.offline.len() > 1024 - intervals
                || rule.profile.faults.len() > 1024 - faults
            {
                return Err(Error::ResourceLimit);
            }
            intervals += rule.profile.offline.len();
            faults += rule.profile.faults.len();
            rule.profile.normalize(&self.limits)?;
        }
        if encode(&self)?.len() > 1024 * 1024 {
            return Err(Error::ResourceLimit);
        }
        let expanded = self.expand()?;
        Ok((self, expanded))
    }

    fn expand(&self) -> Result<BTreeMap<RouteKey, usize>, Error> {
        let rules: BTreeMap<_, _> = self
            .rules
            .iter()
            .enumerate()
            .map(|(i, r)| (r.key(), i))
            .collect();
        let mut expanded = BTreeMap::new();
        for source in &self.endpoints {
            for destination in &self.endpoints {
                let src = source.binding.endpoint;
                let dst = destination.binding.endpoint;
                if src == dst {
                    continue;
                }
                for allowed in &source.send {
                    if permit(&destination.receive, allowed.channel).is_ok() {
                        let key = (src, dst, allowed.channel);
                        expanded.insert(key, select_profile(&rules, key)?);
                    }
                }
            }
        }
        Ok(expanded)
    }

    pub(crate) fn endpoint(&self, endpoint: Endpoint) -> Result<&EndpointRegistration, Error> {
        let index = self
            .endpoints
            .binary_search_by_key(&endpoint, |e| e.binding.endpoint)
            .map_err(|_| Error::UnknownEndpoint)?;
        Ok(&self.endpoints[index])
    }

    fn channel(&self, channel: ChannelId) -> Result<&ChannelSpec, Error> {
        let index = self
            .channels
            .binary_search_by_key(&channel, |c| c.id)
            .map_err(|_| Error::UnknownChannel)?;
        Ok(&self.channels[index])
    }

    pub(crate) fn check_header(&self, header: &MessageHeader, reliable: bool) -> Result<(), Error> {
        if header.run != self.run || header.config != self.config {
            return Err(Error::ConfigMismatch);
        }
        if header.source == header.destination {
            return Err(Error::SelfRoute);
        }
        let source = self.endpoint(header.source)?;
        let destination = self.endpoint(header.destination)?;
        let channel = self.channel(header.channel)?;
        let send = permit(&source.send, header.channel)?;
        let receive = permit(&destination.receive, header.channel)?;
        if reliable && channel.reliability != Reliability::Reliable {
            return Err(Error::PermissionDenied);
        }
        match (channel.context, header.context) {
            (ContextPolicy::Forbidden, None) => {}
            (ContextPolicy::Required, Some(id))
                if id != [0; 32]
                    && send.contexts.binary_search(&id).is_ok()
                    && receive.contexts.binary_search(&id).is_ok() => {}
            _ => return Err(Error::ContextMismatch),
        }
        if header.codec == [0; 32] {
            return Err(Error::InvalidConfig);
        }
        if header.origin.step_id == 0 || header.origin.source_incarnation == 0 {
            return Err(Error::InvalidOrigin);
        }
        if header.logical_id != header.hash_id() {
            return Err(Error::HashMismatch);
        }
        Ok(())
    }

    pub(crate) fn check_message(&self, message: &Message, reliable: bool) -> Result<(), Error> {
        if message.payload.len() as u64 > self.limits.payload_bytes {
            return Err(Error::PayloadTooLarge);
        }
        self.check_header(&message.header(), reliable)?;
        if message.payload_hash != crate::payload_hash(&message.payload) {
            return Err(Error::HashMismatch);
        }
        Ok(())
    }
}

fn select_profile(
    rules: &BTreeMap<RuleKey, usize>,
    (src, dst, channel): RouteKey,
) -> Result<usize, Error> {
    let selectors = |e: Endpoint| {
        [
            EndpointSelector::Any,
            EndpointSelector::Kind(e.kind()),
            EndpointSelector::Exact(e),
        ]
    };
    let mut best = None;
    let mut ambiguous = false;
    for (s, src) in selectors(src).into_iter().enumerate() {
        for (d, dst) in selectors(dst).into_iter().enumerate() {
            for (c, channel) in [ChannelSelector::Any, ChannelSelector::Exact(channel)]
                .into_iter()
                .enumerate()
            {
                let Some(&index) = rules.get(&(src, dst, channel)) else {
                    continue;
                };
                let priority = (
                    usize::from(s == 2) + usize::from(d == 2),
                    usize::from(s == 1) + usize::from(d == 1),
                    c,
                );
                match best {
                    Some((p, _)) if p > priority => {}
                    Some((p, _)) if p == priority => ambiguous = true,
                    _ => {
                        best = Some((priority, index));
                        ambiguous = false;
                    }
                }
            }
        }
    }
    if ambiguous {
        return Err(Error::AmbiguousProfile);
    }
    best.map(|(_, index)| index).ok_or(Error::MissingProfile)
}
