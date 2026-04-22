import React, { useState } from 'react';

export const ChatPanel: React.FC = () => {
  const [messages, setMessages] = useState([
    { id: '1', role: 'Planner', text: 'I have analyzed the project. Requirements look good. Starting PRD generation.', time: '10:05 AM' },
    { id: '2', role: 'System', text: 'Workflow started.', time: '10:04 AM' },
  ]);
  const [input, setInput] = useState('');

  const handleSend = () => {
    if (!input.trim()) return;
    setMessages([...messages, { 
      id: Date.now().toString(), 
      role: 'User', 
      text: input, 
      time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) 
    }]);
    setInput('');
  };

  return (
    <div className="chat-panel">
      <div className="chat-panel__messages">
        {messages.map((msg) => (
          <div key={msg.id} className={`chat-message chat-message--${msg.role.toLowerCase()}`}>
            <header className="chat-message__header">
              <span className="chat-message__role">{msg.role}</span>
              <span className="chat-message__time">{msg.time}</span>
            </header>
            <div className="chat-message__text">{msg.text}</div>
          </div>
        ))}
      </div>
      
      <div className="chat-panel__input">
        <input 
          type="text" 
          placeholder="Talk to the team..." 
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSend()}
        />
        <button onClick={handleSend}>Send</button>
      </div>
    </div>
  );
};
