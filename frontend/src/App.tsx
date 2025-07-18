import React, { useState, useRef, useEffect } from 'react';
import Peer from "simple-peer";

export default function App() {
  const [ws, setWs] = useState(null);
  const [signalData, setSignalData] = useState('');
  const peerRef = useRef();

  useEffect(() => {
    const socket = new WebSocket('ws://localhost:3030/ws');
    socket.onmessage = (ev) => {
      const msg = JSON.parse(ev.data);
      peerRef.current.signal(msg);
    };
    setWs(socket);

    const p = new Peer({ initiator: true, trickle: false });
    p.on('signal', (data) => {
      socket.send(JSON.stringify(data));
      setSignalData(JSON.stringify(data));
    });
    p.on('connect', () => console.log('P2P connected'));
    p.on('data', (data) => console.log('Received:', data));

    peerRef.current = p;
  }, []);

  const sendTest = () => {
    peerRef.current?.send('Hello from peer');
  };

  return (
    <div style={{ padding: '1rem' }}>
      <h1>CrystalPeer</h1>
      <button onClick={sendTest}>Send Test</button>
      <pre>{signalData}</pre>
    </div>
  );
}