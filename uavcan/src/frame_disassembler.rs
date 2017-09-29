use bit_field::BitField;

use {
    TailByte,
    TransferFrame,
    Frame,
    Header,
};

use serializer::*;



pub struct FrameDisassembler<F: Frame> {
    serializer: Serializer<F::Body>,
    started: bool,
    finished: bool,
    id: u32,
    toggle: bool,
    transfer_id: u8,
}

impl<F: Frame> FrameDisassembler<F> {
    pub fn from_uavcan_frame(frame: F, transfer_id: u8) -> Self {
        let (header, body) = frame.to_parts();
        Self{
            serializer: Serializer::from_structure(body),
            started: false,
            finished: false,
            id: header.id(),
            toggle: false,
            transfer_id: transfer_id,
        }
    }

    pub fn finished(&self) -> bool { self.finished }
    
    pub fn next_transfer_frame<T: TransferFrame>(&mut self) -> Option<T> {
        let max_data_length = T::max_data_length();
        let mut transport_frame = T::with_length(self.id, max_data_length);
        
        let first_of_multi_frame = !self.started && !self.serializer.single_frame_transfer();

        if self.finished {
            return None;
        } else if first_of_multi_frame {
            let crc = self.serializer.crc(F::DATA_TYPE_SIGNATURE);
            transport_frame.data_as_mut()[0] = crc.get_bits(0..8) as u8;
            transport_frame.data_as_mut()[1] = crc.get_bits(8..16) as u8;
            {
                let mut buffer = SerializationBuffer::with_empty_buffer(&mut transport_frame.data_as_mut()[2..max_data_length-1]);
                self.serializer.serialize(&mut buffer);
            }
            transport_frame.data_as_mut()[max_data_length-1] = TailByte::new(!self.started, false, self.toggle, self.transfer_id).into();
        } else {
            let (frame_length, end_of_transfer) = {
                let mut buffer = SerializationBuffer::with_empty_buffer(&mut transport_frame.data_as_mut()[0..max_data_length-1]);
                if SerializationResult::Finished == self.serializer.serialize(&mut buffer){
                    self.finished = true;
                    ((buffer.bit_length()+7)/8 + 1, true)
                } else {
                    (max_data_length, false)
                }
            };
            transport_frame.set_data_length(frame_length);
            transport_frame.data_as_mut()[frame_length-1] = TailByte::new(!self.started, end_of_transfer, self.toggle, self.transfer_id).into();
        }
        
        self.started = true;
        self.toggle = !self.toggle;
        
        return Some(transport_frame);
    }
}




#[cfg(test)]
mod tests {

    use uavcan;
    
    use bit_field::BitField;
    
    use tests::{
        CanFrame,
        CanID,
    };
    
    use *;
    use types::*;
    use frame_disassembler::*;

    
    #[test]
    fn serialize_node_status_frame() {

        #[derive(UavcanStruct, Default)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        message_frame_header!(NodeStatusHeader, 341);
        
        uavcan_frame!(NodeStatusMessage, NodeStatusHeader, NodeStatus, 0);
            
        let can_frame = CanFrame{id: CanID(NodeStatusHeader::new(0, 32).id()), dlc: 8, data: [1, 0, 0, 0, 0b10011100, 5, 0, TailByte{start_of_transfer: true, end_of_transfer: true, toggle: false, transfer_id: 0}.into()]};

        let uavcan_frame = NodeStatusMessage{
            header: NodeStatusHeader::new(0, 32),
            body: NodeStatus{
                uptime_sec: 1.into(),
                health: 2.into(),
                mode: 3.into(),
                sub_mode: 4.into(),
                vendor_specific_status_code: 5.into(),
            },
        };

        let mut frame_generator = FrameDisassembler::from_uavcan_frame(uavcan_frame, 0);

        assert_eq!(frame_generator.next_transfer_frame(), Some(can_frame));
        assert_eq!(frame_generator.next_transfer_frame::<CanFrame>(), None);
        
    }
    
    #[test]
    fn serialize_multi_frame() {

        #[derive(UavcanStruct)]
        struct LogLevel {
            value: Uint3,
        }
        
        #[derive(UavcanStruct)]
        struct LogMessage {
            level: LogLevel,
            source: DynamicArray31<Uint8>,
            text: DynamicArray90<Uint8>,
        }

        message_frame_header!(LogMessageHeader, 16383);

        uavcan_frame!(LogMessageMessage, LogMessageHeader, LogMessage, 0xd654a48e0c049d75);
        
        let uavcan_frame = LogMessageMessage{
            header: LogMessageHeader::new(0, 32),
            body: LogMessage{
                level: LogLevel{value: 0.into()},
                source: DynamicArray31::with_str("test source"),
                text: DynamicArray90::with_str("test text"),
            },
        };

        assert_eq!(LogMessage::FLATTENED_FIELDS_NUMBER, 3);
        
        let mut frame_generator = FrameDisassembler::from_uavcan_frame(uavcan_frame, 0);

        let crc = frame_generator.serializer.crc(0xd654a48e0c049d75);

        
        assert_eq!(
            frame_generator.next_transfer_frame(),
            Some(CanFrame{
                id: CanID(LogMessageHeader::new(0, 32).id()),
                dlc: 8,
                data: [crc.get_bits(0..8) as u8, crc.get_bits(8..16) as u8, 0u8.set_bits(0..5, 11).set_bits(5..8, 0).get_bits(0..8), b't', b'e', b's', b't', TailByte{start_of_transfer: true, end_of_transfer: false, toggle: false, transfer_id: 0}.into()],
            })
        );
        
        assert_eq!(
            frame_generator.next_transfer_frame(),
            Some(CanFrame{
                id: CanID(LogMessageHeader::new(0, 32).id()),
                dlc: 8,
                data: [b' ', b's', b'o', b'u', b'r', b'c', b'e', TailByte{start_of_transfer: false, end_of_transfer: false, toggle: true, transfer_id: 0}.into()],
            })
        );
        
        assert_eq!(
            frame_generator.next_transfer_frame(),
            Some(CanFrame{
                id: CanID(LogMessageHeader::new(0, 32).id()),
                dlc: 8,
                data: [b't', b'e', b's', b't', b' ', b't', b'e', TailByte{start_of_transfer: false, end_of_transfer: false, toggle: false, transfer_id: 0}.into()],
            })
        );
        
        assert_eq!(
            frame_generator.next_transfer_frame(),
            Some(CanFrame{
                id: CanID(LogMessageHeader::new(0, 32).id()),
                dlc: 3,
                data: [b'x', b't', TailByte{start_of_transfer: false, end_of_transfer: true, toggle: true, transfer_id: 0}.into(), 0, 0, 0, 0, 0],
            })
        );

        assert_eq!(frame_generator.next_transfer_frame::<CanFrame>(), None);
       
    }

}
