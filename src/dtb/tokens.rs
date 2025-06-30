// ABOUTME: DTB structure block token definitions and parsing
// ABOUTME: Handles the four basic DTB tokens with 4-byte alignment

use super::error::DtbError;

/// DTB token constants as defined in the device tree specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DtbToken {
    /// Begin node token (0x00000001)
    BeginNode,
    /// End node token (0x00000002)
    EndNode,
    /// Property token (0x00000003)
    Property,
    /// End of structure token (0x00000009)
    End,
}

impl DtbToken {
    /// Begin node token constant
    pub const FDT_BEGIN_NODE: u32 = 0x00000001;
    /// End node token constant
    pub const FDT_END_NODE: u32 = 0x00000002;
    /// Property token constant
    pub const FDT_PROP: u32 = 0x00000003;
    /// End of structure token constant
    pub const FDT_END: u32 = 0x00000009;

    /// Convert u32 value to DtbToken
    pub fn from_u32(value: u32) -> Result<Self, DtbError> {
        match value {
            Self::FDT_BEGIN_NODE => Ok(DtbToken::BeginNode),
            Self::FDT_END_NODE => Ok(DtbToken::EndNode),
            Self::FDT_PROP => Ok(DtbToken::Property),
            Self::FDT_END => Ok(DtbToken::End),
            _ => Err(DtbError::InvalidToken),
        }
    }

    /// Convert DtbToken to u32 value
    pub fn to_u32(self) -> u32 {
        match self {
            DtbToken::BeginNode => Self::FDT_BEGIN_NODE,
            DtbToken::EndNode => Self::FDT_END_NODE,
            DtbToken::Property => Self::FDT_PROP,
            DtbToken::End => Self::FDT_END,
        }
    }

    /// Parse a single token from input bytes with 4-byte alignment
    pub fn parse(input: &[u8]) -> Result<(&[u8], Self), DtbError> {
        if input.len() < 4 {
            return Err(DtbError::MalformedHeader);
        }

        // Ensure 4-byte alignment
        if (input.as_ptr() as usize) % 4 != 0 {
            return Err(DtbError::AlignmentError);
        }

        // Parse token value using array slicing
        let token_bytes: [u8; 4] = input[0..4]
            .try_into()
            .map_err(|_| DtbError::MalformedHeader)?;
        let token_value = u32::from_be_bytes(token_bytes);

        let token = Self::from_u32(token_value)?;
        Ok((&input[4..], token))
    }

    /// Calculate padding needed for 4-byte alignment
    pub fn calculate_padding(offset: usize) -> usize {
        (4 - (offset % 4)) % 4
    }

    /// Skip padding bytes to achieve 4-byte alignment
    pub fn skip_padding(input: &[u8], current_offset: usize) -> &[u8] {
        let padding = Self::calculate_padding(current_offset);
        if padding > 0 && input.len() >= padding {
            &input[padding..]
        } else {
            input
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_constants() {
        assert_eq!(DtbToken::FDT_BEGIN_NODE, 0x00000001);
        assert_eq!(DtbToken::FDT_END_NODE, 0x00000002);
        assert_eq!(DtbToken::FDT_PROP, 0x00000003);
        assert_eq!(DtbToken::FDT_END, 0x00000009);
    }

    #[test]
    fn test_token_from_u32() {
        assert_eq!(DtbToken::from_u32(0x00000001).unwrap(), DtbToken::BeginNode);
        assert_eq!(DtbToken::from_u32(0x00000002).unwrap(), DtbToken::EndNode);
        assert_eq!(DtbToken::from_u32(0x00000003).unwrap(), DtbToken::Property);
        assert_eq!(DtbToken::from_u32(0x00000009).unwrap(), DtbToken::End);

        assert!(DtbToken::from_u32(0x12345678).is_err());
    }

    #[test]
    fn test_token_to_u32() {
        assert_eq!(DtbToken::BeginNode.to_u32(), 0x00000001);
        assert_eq!(DtbToken::EndNode.to_u32(), 0x00000002);
        assert_eq!(DtbToken::Property.to_u32(), 0x00000003);
        assert_eq!(DtbToken::End.to_u32(), 0x00000009);
    }

    #[test]
    fn test_token_parse_begin_node() {
        let data = [0x00, 0x00, 0x00, 0x01, 0x12, 0x34, 0x56, 0x78];
        let result = DtbToken::parse(&data);
        assert!(result.is_ok());
        let (remaining, token) = result.unwrap();
        assert_eq!(token, DtbToken::BeginNode);
        assert_eq!(remaining, &[0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_token_parse_property() {
        let data = [0x00, 0x00, 0x00, 0x03, 0xAB, 0xCD, 0xEF, 0x00];
        let result = DtbToken::parse(&data);
        assert!(result.is_ok());
        let (remaining, token) = result.unwrap();
        assert_eq!(token, DtbToken::Property);
        assert_eq!(remaining, &[0xAB, 0xCD, 0xEF, 0x00]);
    }

    #[test]
    fn test_token_parse_invalid() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let result = DtbToken::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_parse_insufficient_data() {
        let data = [0x00, 0x00, 0x00];
        let result = DtbToken::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_padding() {
        assert_eq!(DtbToken::calculate_padding(0), 0);
        assert_eq!(DtbToken::calculate_padding(1), 3);
        assert_eq!(DtbToken::calculate_padding(2), 2);
        assert_eq!(DtbToken::calculate_padding(3), 1);
        assert_eq!(DtbToken::calculate_padding(4), 0);
        assert_eq!(DtbToken::calculate_padding(5), 3);
    }

    #[test]
    fn test_skip_padding() {
        let data = [0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05];

        // No padding needed at offset 0
        let result = DtbToken::skip_padding(&data, 0);
        assert_eq!(result, &data);

        // 3 bytes padding needed at offset 1
        let result = DtbToken::skip_padding(&data[1..], 1);
        assert_eq!(result, &data[4..]);

        // 2 bytes padding needed at offset 2
        let result = DtbToken::skip_padding(&data[2..], 2);
        assert_eq!(result, &data[4..]);
    }
}
