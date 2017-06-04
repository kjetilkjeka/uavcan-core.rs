use uavcan_frame::TransportFrame;

pub enum CanID {
    Extended(u32),
    Normal(u16),
}

pub struct CanFrame {
    pub id: CanID,
    pub dlc: usize,
    pub data: [u8; 8],
}

pub trait ToCanID {
    fn to_can_id(&self) -> CanID;
}

impl TransportFrame for CanFrame {
    fn with_data(id: u32, data: &[u8]) -> CanFrame {
        let mut can_data = [0; 8];
        can_data[0..data.len()].clone_from_slice(data);
        CanFrame{id: CanID::Extended(id), dlc: data.len(), data: can_data}
    }

    fn get_max_data_length() -> usize {
        8
    }

    fn get_data(&self) -> &[u8] {
        &self.data[0..self.dlc]
    }

    fn get_id(&self) -> u32 {
        match self.id {
            CanID::Extended(x) => x,
            CanID::Normal(x) => x as u32,
        }
    }
}
            
