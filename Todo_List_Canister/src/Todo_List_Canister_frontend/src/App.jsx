import React, { useState, useEffect } from 'react';
import { HttpAgent, Actor } from '@dfinity/agent';
import { AuthClient } from '@dfinity/auth-client';
import { canisterId, idlFactory } from '../../declarations/Todo_List_Canister_backend';

const App = () => {
  const [authClient, setAuthClient] = useState(null);
  const [actor, setActor] = useState(null);
  const [todos, setTodos] = useState([]);
  const [taskName, setTaskName] = useState('');
  const [doneBy, setDoneBy] = useState('');
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [principal, setPrincipal] = useState(null);

  // Initialize AuthClient on load
  useEffect(() => {
    (async () => {
      const client = await AuthClient.create();
      setAuthClient(client);
      if (await client.isAuthenticated()) {
        await handleAuthenticated(client);
      }
    })();
  }, []);

  // Set up actor and fetch data after authentication
  const handleAuthenticated = async (client) => {
    const identity = client.getIdentity();
    const agent = new HttpAgent({ identity });
    if (process.env.DFX_NETWORK !== 'ic') {
      await agent.fetchRootKey(); // For local dev
    }
    const actor = Actor.createActor(idlFactory, { agent, canisterId });
    setActor(actor);
    setIsAuthenticated(true);
    const principal = await actor.whoami();
    setPrincipal(principal);
    fetchTodos(actor);
  };
  const login = async () => {
    const internetIdentityCanisterId = 'cgpjn-omaaa-aaaaa-qaakq-cai'; // Hardcode for now
    await authClient.login({
      identityProvider: process.env.DFX_NETWORK === 'ic'
        ? 'https://identity.ic0.app'
        : `http://${internetIdentityCanisterId}.localhost:4943/`, // Recommended URL
      onSuccess: () => handleAuthenticated(authClient),
    });
  };
  // // Login with Internet Identity
  // const login = async () => {
  //   await authClient.login({
  //     identityProvider: process.env.DFX_NETWORK === 'ic'
  //       ? 'https://identity.ic0.app'
  //       : `http://localhost:4943?canisterId=${process.env.CANISTER_ID_INTERNET_IDENTITY}`,
  //     onSuccess: () => handleAuthenticated(authClient),
  //   });
  // };

  // Logout
  const logout = async () => {
    await authClient.logout();
    setIsAuthenticated(false);
    setPrincipal(null);
    setTodos([]);
    setActor(null);
  };

  // Fetch todos
  const fetchTodos = async (actorInstance = actor) => {
    if (actorInstance) {
      const todos = await actorInstance.get_todos();
      setTodos(todos);
    }
  };

  // Add a todo
  const addTodo = async () => {
    if (taskName && doneBy && actor) {
      await actor.add_todo(taskName, doneBy);
      setTaskName('');
      setDoneBy('');
      fetchTodos();
    }
  };

  // Toggle todo completion
  const toggleTodo = async (index) => {
    if (actor) {
      await actor.toggle_todo(BigInt(index));
      fetchTodos();
    }
  };

  return (
    <div style={{ padding: '20px' }}>
      <h1>Web3 To-Do List</h1>
      {!isAuthenticated ? (
        <button onClick={login}>Login with Internet Identity</button>
      ) : (
        <>
          <p>Your Principal: {principal ? principal.toText() : 'Loading...'}</p>
          <button onClick={logout}>Logout</button>
          <div>
            <input
              type="text"
              value={taskName}
              onChange={(e) => setTaskName(e.target.value)}
              placeholder="Task name"
            />
            <input
              type="text"
              value={doneBy}
              onChange={(e) => setDoneBy(e.target.value)}
              placeholder="Done by"
            />
            <button onClick={addTodo}>Add Todo</button>
          </div>
          <ul>
            {todos.map((todo, index) => (
              <li key={index}>
                <input
                  type="checkbox"
                  checked={todo[2]} // completed
                  onChange={() => toggleTodo(index)}
                />
                {todo[0]} - Done by: {todo[1]} {/* name, doneby */}
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
};

export default App;