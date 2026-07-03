use cpal::SizedSample;
use strided::MutStride;

pub struct AudioBuffer<'a, T>
where
    T: SizedSample,
{
    frames: usize,
    channels: u16,
    interleaved: &'a mut [T],
}

impl<'a, T> AudioBuffer<'a, T>
where
    T: SizedSample,
{
    pub fn wrap(data: &'a mut [T], channels: u16) -> Self {
        assert_eq!(
            data.len() % channels as usize,
            0,
            "data.len() isn't an exact multiple of channels!"
        );
        let frames = data.len() / channels as usize;
        Self { frames, channels, interleaved: data }
    }

    pub fn channel_mut(&mut self, ch: u16) -> MutStride<'_, T> {
        assert!(ch < self.channels, "channel index out of bounds");
        let all = MutStride::new(&mut *self.interleaved);
        all.substrides_mut(self.channels as usize)
            .nth(ch as usize)
            .expect("substrides_mut always yields `channels` slices")
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn interleaved(&mut self) -> &mut [T] {
        self.interleaved
    }

    pub fn to_planar(&mut self) -> PlanarAudioBuffer<T>
    where
        T: Default,
    {
        let mut planar = PlanarAudioBuffer::new(self.channels, self.frames);
        for ch in 0..self.channels {
            let src = self.channel_mut(ch);
            let dst = planar.channel_mut(ch);
            for (s, d) in src.into_iter().zip(dst.iter_mut()) {
                *d = *s;
            }
        }
        planar
    }
}

pub struct PlanarAudioBuffer<T> {
    channels: u16,
    frames: usize,
    data: Vec<T>,
    ptrs: Vec<*mut T>,
}

impl<T> PlanarAudioBuffer<T> {
    pub fn channel_mut(&mut self, ch: u16) -> &mut [T] {
        let ch = ch as usize;
        &mut self.data[ch * self.frames..(ch + 1) * self.frames]
    }

    pub fn channels(&self) -> u16 {
        self.channels
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn as_ptr(&mut self) -> *mut *mut T {
        self.ptrs.as_mut_ptr()
    }
}

impl<T: Default + Clone> PlanarAudioBuffer<T> {
    pub fn new(channels: u16, frames: usize) -> Self {
        let mut data = vec![T::default(); channels as usize * frames];
        let ptrs = data.chunks_mut(frames).map(|c| c.as_mut_ptr()).collect();
        Self { channels, frames, data, ptrs }
    }
}

impl<T: SizedSample> PlanarAudioBuffer<T> {
    pub fn write_into_interleaved<'a>(&self, interleaved: &mut AudioBuffer<'a, T>) {
        for ch in 0..self.channels {
            let src_start = ch as usize * self.frames;
            let src = &self.data[src_start..src_start + self.frames];
            let dst = interleaved.channel_mut(ch);
            for (d, s) in dst.into_iter().zip(src.iter()) {
                *d = *s;
            }
        }
    }
}

