import { config } from "../config/Config";

const baseURL = config.serverURL;

export type ServerList = {
  name: string;
  players: number;
  seed: number;
};

export type RankingResponse = { name: string; kills: number; deaths: number }[];

type GetIDResponse =
  | {
      Ok: number;
    }
  | {
      Err: string;
    };

export const ServerRequests = {
  async getServerList(): Promise<ServerList[]> {
    const req = await fetch(`${baseURL}/get_server_list`);
    const body = await req.json();
    return body;
  },

  async sendError(error: Error) {
    try {
      const stack = error.stack;
      const body = JSON.stringify({ error: error.message, stack });
      await fetch(`${baseURL}/error`, {
        method: "POST",
        body,
        headers: { "Content-Type": "application/json" },
      });
    } catch (e) {
      console.error("Failed to send error to server");
    }
  },

  async getRanking(): Promise<RankingResponse> {
    const req = await fetch(`${baseURL}/ranking`);
    const body = await req.json();
    return body;
  },

  async getPlayerID(serverName: string): Promise<number> {
    const search = new URLSearchParams();
    search.append("server_id", serverName);
    const url = new URL(`${baseURL}/get_player_id?${search.toString()}`);
    const req = await fetch(url);
    const body: GetIDResponse = await req.json();

    if ("Ok" in body) {
      return body.Ok;
    }

    throw new Error(body.Err);
  },
};
