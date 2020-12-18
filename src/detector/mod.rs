use crate::detector::internals::Pitch;
use crate::float::Float;

pub mod autocorrelation;
pub mod internals;
pub mod mcleod;

pub trait PitchDetector<T>
where
    T: Float,
{
    fn get_pitch<'a, S, I>(
        &mut self,
        signal: S,
        sample_rate: usize,
        power_threshold: T,
        clarity_threshold: T,
    ) -> Option<Pitch<T>>
    where
        I: ExactSizeIterator<Item = &'a T>,
        S: IntoIterator<IntoIter = I, Item = &'a T> + Copy;
}
