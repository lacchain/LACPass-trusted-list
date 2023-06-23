use super::data_interface::DidDataInterfaceService;

pub struct DidService {
    pub did_data_interface_service: DidDataInterfaceService,
}

impl DidService {
    pub fn new() -> DidService {
        DidService {
            did_data_interface_service: DidDataInterfaceService {},
        }
    }
}
