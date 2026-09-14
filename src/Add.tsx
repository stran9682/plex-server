import "./App.css"
import { invoke } from '@tauri-apps/api/core';
import { useState } from "react";

function Add() {
    const [ticket, setTicket] = useState<string>("")

    const importTicket = (ticket: string) => {
        invoke('import_ticket', {ticket: ticket})
            .catch((error) => console.error(error))
    }

    return <div>
        <input 
            type="text"
            value={ticket} 
            onChange={(e) => setTicket(e.target.value)} 
        />

        <button disabled= {ticket === ""} onClick={() => importTicket(ticket)}>
            Import ticket
        </button>
    </div>
}

export default Add