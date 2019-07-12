#[cfg(feature = "repr-prost")]
#[macro_use]
extern crate quick_error;

/// Unifies different interfaces of message in different protocol implementations.
pub trait GenericMessage {
    type Error;

    /// Get the size of encoded messages.
    fn compute_size(&self) -> usize;
    /// Encode the message into buf.
    fn encode_into(&self, buf: &mut Vec<u8>) -> Result<(), Self::Error>;
    /// Decode a message from the data.
    fn decode_from(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}

pub trait GenericEnum: Sized {
    fn values() -> &'static [Self];
}

#[cfg(feature = "repr-protobuf")]
mod codec {
    pub use protobuf::ProtobufError;

    impl<T: protobuf::Message> super::GenericMessage for T {
        type Error = ProtobufError;

        #[inline]
        fn compute_size(&self) -> usize {
            protobuf::Message::compute_size(self) as usize
        }

        #[inline]
        fn encode_into(&self, buf: &mut Vec<u8>) -> Result<(), ProtobufError> {
            protobuf::Message::write_to_vec(self, buf)
        }

        #[inline]
        fn decode_from(&mut self, data: &[u8]) -> Result<(), ProtobufError> {
            protobuf::Message::merge_from_bytes(self, data)
        }
    }

    impl<T: protobuf::ProtobufEnum> super::GenericEnum for T {
        #[inline]
        fn values() -> &'static [Self] {
            <T as protobuf::ProtobufEnum>::values()
        }
    }
}

#[cfg(feature = "repr-prost")]
mod codec {
    use prost::{EncodeError, DecodeError};

    quick_error! {
        /// The error for PROST!. It defines error in a weird way.
        #[derive(Debug, PartialEq)]
        pub enum ProtobufError {
            /// Error for when encoding messages.
            Encode(err: EncodeError) {
                from()
                cause(err)
                description(err.description())
                display("{:?}", err)
            }
            /// Error for decoding messages.
            Decode(err: DecodeError) {
                from()
                cause(err)
                description(err.description())
                display("{:?}", err)
            }
        }
    }

    impl<T: prost::Message> super::GenericMessage for T {
        type Error = ProtobufError;

        #[inline]
        fn compute_size(&self) -> usize {
            self.encoded_len()
        }

        #[inline]
        fn encode_into(&self, data: &mut Vec<u8>) -> Result<(), ProtobufError> {
            prost::Message::encode(self, data).map_err(ProtobufError::Encode)
        }

        #[inline]
        fn decode_from(&mut self, data: &[u8]) -> Result<(), ProtobufError> {
            prost::Message::merge(self, data).map_err(ProtobufError::Decode)
        }
    }
}

pub use codec::ProtobufError;
#[cfg(feature = "repr-prost")]
pub use jinkela_derive::*;
