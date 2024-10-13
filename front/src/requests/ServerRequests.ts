import { config } from "../config/Config";

const url = config.serverURL;

export type ServerList = {
  name: string;
  players: number;
};

export const ServerRequests = {
  async getServerList(): Promise<ServerList[]> {
    const req = await fetch(`${url}/get_server_list`);
    const body = await req.json();
    return body;
  },
};
