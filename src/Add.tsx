import { useRef, useState } from "react";
import "./Add.css";
import { invoke } from "@tauri-apps/api/core";

enum Popup {
	Ticket,
	Local,
	Remote,
}

function TicketPopup({ setPopup }: { setPopup: () => void }) {
	const [ticket, setTicket] = useState<string>("");

	const importTicket = (ticket: string) => {
		invoke("import_ticket", { ticket: ticket }).catch((error) =>
			console.error(error),
		);
	};

	return (
		<div className="popup">
			<div className="popup-container">
				<h2>Add Ticket</h2>
				<input
					type="text"
					value={ticket}
					onChange={(e) => setTicket(e.target.value)}
				/>

				<div className="popup-actions">
					<button disabled={ticket === ""} onClick={() => importTicket(ticket)}>
						Add
					</button>
					<button onClick={() => setPopup()}>Close</button>
				</div>
			</div>
		</div>
	);
}

function RemotePopup({ setPopup }: { setPopup: () => void }) {
	const [endpoint, setEndpoint] = useState<string>("");
	const [namespace, setNamespace] = useState<string>("");

	const importRemote = (endpoint: string, namespace: string) => {
		invoke("add_remote_store", { endpoint: endpoint, namespace: namespace });
	};

	return (
		<div className="popup">
			<div className="popup-container">
				<h2>Add Remote</h2>

				<h3>Endpoint</h3>
				<input
					type="text"
					value={endpoint}
					onChange={(e) => setEndpoint(e.target.value)}
				/>

				<h3>Namespace</h3>
				<input
					type="text"
					value={namespace}
					onChange={(e) => setNamespace(e.target.value)}
				/>

				<div className="popup-actions">
					<button onClick={() => importRemote(endpoint, namespace)}>Add</button>
					<button onClick={() => setPopup()}>Close</button>
				</div>
			</div>
		</div>
	);
}

function LocalPopup({ setPopup }: { setPopup: () => void }) {
	const [filePath, setFilePath] = useState<string | string[] | null>(null);
	const [error, setError] = useState<string | null>(null);

	return (
		<div className="popup">
			<div className="popup-container">
				<div className="popup-actions">
					<button>Add</button>
					<button onClick={() => setPopup()}>Close</button>
				</div>
			</div>
		</div>
	);
}

function Add() {
	const [popup, setPopup] = useState<Popup | null>(null);

	const renderPopup = () => {
		if (popup === null) {
			return;
		}

		const handleClose = () => setPopup(null);

		switch (popup) {
			case Popup.Ticket:
				return <TicketPopup setPopup={handleClose} />;
			case Popup.Local:
				return <LocalPopup setPopup={handleClose} />;
			case Popup.Remote:
				return <RemotePopup setPopup={handleClose} />;
		}
	};

	return (
		<div className="menu">
			<h1>Add</h1>

			{popup !== null && renderPopup()}

			<div className="menu-item">
				<div className="menu-label">
					<h3>Import Ticket</h3>
					Sync a store with a peer
				</div>

				<button onClick={() => setPopup(Popup.Ticket)}>Add</button>
			</div>

			<div className="menu-item">
				<div className="menu-label">
					<h3>Add video from local</h3>
					Create a syncable store, or add to an existing one
				</div>

				<button onClick={() => setPopup(Popup.Local)}>Add</button>
			</div>

			<div className="menu-item">
				<h3>Add a remote</h3>

				<button onClick={() => setPopup(Popup.Remote)}>Add</button>
			</div>
		</div>
	);
}

export default Add;
