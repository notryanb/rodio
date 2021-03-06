//! Decodes samples from an audio file.

use std::error::Error;
use std::fmt;
use std::io::{Read, Seek};
use std::time::Duration;

use Source;

#[cfg(feature = "flac")]
mod flac;
#[cfg(feature = "vorbis")]
mod vorbis;
#[cfg(feature = "wav")]
mod wav;

/// Source of audio samples from decoding a file.
///
/// Supports WAV, Vorbis and Flac.
#[cfg(any(feature = "wav", feature = "flac", feature = "vorbis"))]
pub struct Decoder<R>(DecoderImpl<R>) where R: Read + Seek;

#[cfg(not(any(feature = "wav", feature = "flac", feature = "vorbis")))]
pub struct Decoder<R>(::std::marker::PhantomData<R>);

#[cfg(any(feature = "wav", feature = "flac", feature = "vorbis"))]
enum DecoderImpl<R>
    where R: Read + Seek
{
    #[cfg(feature = "wav")]
    Wav(wav::WavDecoder<R>),
    #[cfg(feature = "vorbis")]
    Vorbis(vorbis::VorbisDecoder<R>),
    #[cfg(feature = "flac")]
    Flac(flac::FlacDecoder<R>),
}

impl<R> Decoder<R>
    where R: Read + Seek + Send + 'static
{
    /// Builds a new decoder.
    ///
    /// Attempts to automatically detect the format of the source of data.
    #[allow(unused_variables)]
    pub fn new(data: R) -> Result<Decoder<R>, DecoderError> {
        #[cfg(feature = "wav")]
        let data = match wav::WavDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Wav(decoder)));
            },
        };

        #[cfg(feature = "flac")]
        let data = match flac::FlacDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Flac(decoder)));
            },
        };

        #[cfg(feature = "vorbis")]
        let data = match vorbis::VorbisDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Vorbis(decoder)));
            },
        };

        Err(DecoderError::UnrecognizedFormat)
    }
}

#[cfg(not(any(feature = "wav", feature = "flac", feature = "vorbis")))]
impl<R> Iterator for Decoder<R>
    where R: Read + Seek
{
    type Item = i16;

    fn next(&mut self) -> Option<i16> { None }
}

#[cfg(any(feature = "wav", feature = "flac", feature = "vorbis"))]
impl<R> Iterator for Decoder<R>
    where R: Read + Seek
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref mut source) => source.next(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref mut source) => source.next(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref mut source) => source.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref source) => source.size_hint(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref source) => source.size_hint(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref source) => source.size_hint(),
        }
    }
}

#[cfg(not(any(feature = "wav", feature = "flac", feature = "vorbis")))]
impl<R> Source for Decoder<R>
    where R: Read + Seek
{
    fn current_frame_len(&self) -> Option<usize> { Some(0) }
    fn channels(&self) -> u16 { 0 }
    fn samples_rate(&self) -> u32 { 1 }
    fn total_duration(&self) -> Option<Duration> { Some(Duration::default()) }
}

#[cfg(any(feature = "wav", feature = "flac", feature = "vorbis"))]
impl<R> Source for Decoder<R>
    where R: Read + Seek
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref source) => source.current_frame_len(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref source) => source.current_frame_len(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref source) => source.current_frame_len(),
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref source) => source.channels(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref source) => source.channels(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref source) => source.channels(),
        }
    }

    #[inline]
    fn samples_rate(&self) -> u32 {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref source) => source.samples_rate(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref source) => source.samples_rate(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref source) => source.samples_rate(),
        }
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        match self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(ref source) => source.total_duration(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(ref source) => source.total_duration(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(ref source) => source.total_duration(),
        }
    }
}

/// Error that can happen when creating a decoder.
#[derive(Debug, Clone)]
pub enum DecoderError {
    /// The format of the data has not been recognized.
    UnrecognizedFormat,
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &DecoderError::UnrecognizedFormat => write!(f, "Unrecognized format"),
        }
    }
}

impl Error for DecoderError {
    fn description(&self) -> &str {
        match self {
            &DecoderError::UnrecognizedFormat => "Unrecognized format",
        }
    }
}
