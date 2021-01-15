//! We define `Pitches` an iterator on polyphonic pitches.
use num_complex::Complex;
use num_traits::Zero;
use rustfft::{FFTplanner, FFT};

use super::PitchDetector;
use crate::detector::internals::Pitch;
use crate::float::Float;
use crate::utils::buffer::copy_complex_to_real;
use crate::utils::buffer::copy_real_to_complex;
use crate::utils::buffer::new_complex_buffer;
use crate::utils::buffer::new_real_buffer;
use crate::utils::buffer::ComplexComponent;
use crate::utils::filters::comb_filter;
use std::sync::Arc;

pub struct Pitches<'a, D: PitchDetector<T>, T: Float> {
    detector: &'a mut D,
    remaining_signal: Vec<T>,
    sample_rate: usize,
    power_threshold: T,
    clarity_threshold: T,
    previous_pitch: Option<Pitch<T>>,
    fft: Arc<dyn FFT<T>>,
    inv_fft: Arc<dyn FFT<T>>,
    scratch: Vec<Complex<T>>,
    comb_g: Vec<T>,
    signal_complex: Vec<Complex<T>>,
}

impl<'a, D, T> Pitches<'a, D, T>
where
    D: PitchDetector<T>,
    T: Float,
{
    pub(super) fn new(
        detector: &'a mut D,
        signal: &[T],
        sample_rate: usize,
        power_threshold: T,
        clarity_threshold: T,
    ) -> Self {
        let size = signal.len();
        let signal_complex: Vec<Complex<T>> = new_complex_buffer(size, Complex::zero());
        let scratch: Vec<Complex<T>> = new_complex_buffer(size, Complex::zero());
        let comb_g: Vec<T> = new_real_buffer(signal.len(), T::one());
        let remaining_signal: Vec<T> = signal.to_vec();

        let mut planner = FFTplanner::new(false);
        let fft = planner.plan_fft(size);
        let mut planner = FFTplanner::new(true);
        let inv_fft = planner.plan_fft(size);

        Pitches {
            detector,
            remaining_signal,
            sample_rate,
            power_threshold,
            clarity_threshold,
            previous_pitch: None,
            fft,
            inv_fft,
            signal_complex,
            scratch,
            comb_g,
        }
    }
}

impl<'a, D, T> Iterator for Pitches<'a, D, T>
where
    D: PitchDetector<T>,
    T: Float,
{
    type Item = Pitch<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pitch) = self.previous_pitch.take() {
            let size = self.remaining_signal.len();
            let size_t = T::from_usize(size).unwrap();
            let beta = T::from_f64(50.).unwrap(); // TODO: expose this as a paremeter
            copy_real_to_complex(
                &self.remaining_signal,
                &mut self.signal_complex,
                ComplexComponent::Re,
            );
            self.fft
                .process(&mut self.signal_complex, &mut self.scratch);
            comb_filter(
                pitch.frequency,
                beta,
                size,
                self.sample_rate,
                &mut self.comb_g,
                true,
                true,
                true,
            );

            self.scratch
                .iter_mut()
                .zip(self.comb_g.iter())
                .for_each(|(s_value, c_value)| {
                    s_value.re = s_value.re * (*c_value) / size_t;
                    s_value.im = s_value.im * (*c_value) / size_t;
                });

            self.inv_fft
                .process(&mut self.scratch, &mut self.signal_complex);
            copy_complex_to_real(
                &self.signal_complex,
                &mut self.remaining_signal,
                ComplexComponent::Re,
            );
        }
        self.previous_pitch = self.detector.get_pitch(
            &self.remaining_signal,
            self.sample_rate,
            self.power_threshold,
            self.clarity_threshold,
        );
        self.previous_pitch.clone()
    }
}
