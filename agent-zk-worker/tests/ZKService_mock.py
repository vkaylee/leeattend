class MockZKService:
    def __init__(self, device_id, ip, port):
        self.device_id = device_id
        self.ip = ip
        self.port = port
    
    @property
    def id(self):
        return self.device_id

    def get_device_info(self):
        return {"id": self.device_id, "status": "online"}
