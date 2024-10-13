import { config } from "../config/Config";

const url = config.serverURL;

export const ServerRequests = {
  async getServerList() {
    const req = await fetch(`${url}/get_server_list`);
    const body = await req.json();
    console.log(body);
  },
};
