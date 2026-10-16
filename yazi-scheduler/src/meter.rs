use std::{collections::VecDeque, time::{Duration, Instant}};

const WINDOW: Duration = Duration::from_secs(5);

// Transfer speed over a sliding window. Progress may arrive in bursts, so only changes are recorded,
// and the speed is measured between them.
#[derive(Debug, Default)]
pub(crate) struct Meter {
	samples: VecDeque<(Instant, u64)>,
	last:    Option<Instant>,
}

impl Meter {
	pub(crate) fn sample(&mut self, bytes: u64) {
		let now = Instant::now();
		match self.samples.back() {
			Some(&(_, b)) if b == bytes => {}
			Some(&(at, b)) if now - at > WINDOW => {
				// Resume after a stall from the previous sample, so the idle time is not averaged in
				let prev = self.last.unwrap_or(at);
				self.samples.clear();
				self.samples.extend([(prev, b), (now, bytes)]);
			}
			_ => self.samples.push_back((now, bytes)),
		}

		while self.samples.len() > 2 && now - self.samples[1].0 >= WINDOW {
			self.samples.pop_front();
		}

		self.last = Some(now);
	}

	// Bytes per second: `None` until measurable, zero once progress stops
	pub(crate) fn speed(&self) -> Option<u64> {
		let (&(start, from), &(end, to)) = (self.samples.front()?, self.samples.back()?);
		if self.last? - end > WINDOW {
			return Some(0);
		}

		let secs = (end - start).as_secs_f64();
		(secs >= 1.0).then(|| (to.saturating_sub(from) as f64 / secs) as u64)
	}
}
