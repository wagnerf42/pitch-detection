use crate::detector::internals::Pitch;
use crate::float::Float;

pub mod autocorrelation;
pub mod internals;
pub mod mcleod;
mod polyphonic;
pub use polyphonic::Pitches;

pub trait PitchDetector<T>
where
    T: Float,
{
    fn get_pitch(
        &mut self,
        signal: &[T],
        sample_rate: usize,
        power_threshold: T,
        clarity_threshold: T,
    ) -> Option<Pitch<T>>;

    fn pitches<'a>(
        &'a mut self,
        signal: &[T],
        sample_rate: usize,
        power_threshold: T,
        clarity_threshold: T,
    ) -> Pitches<Self, T>
    where
        Self: Sized,
    {
        Pitches::new(
            self,
            signal,
            sample_rate,
            power_threshold,
            clarity_threshold,
        )
    }
}
