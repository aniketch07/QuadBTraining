import { useState, useEffect } from "react";
import { Actor, HttpAgent } from "@dfinity/agent";
import { idlFactory, canisterId } from "declarations/Todo_auth_ICP_backend";
import { AuthClient } from "@dfinity/auth-client";

function App() {
  const [authClient, setAuthClient] = useState(null);
  const [actor, setActor] = useState(null);
  const [principal, setPrincipal] = useState(null);
  const [tasks, setTasks] = useState([]);
  const [taskName, setTaskName] = useState("");
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    (async () => {
      const client = await AuthClient.create();
      setAuthClient(client);
      if (await client.isAuthenticated()) {
        await handleAuthenticated(client);
      }
    })();
  }, []);

  const handleAuthenticated = async (client) => {
    const identity = client.getIdentity();
    const agent = new HttpAgent({ identity });

    if (process.env.DFX_NETWORK !== "ic") {
      await agent.fetchRootKey();
    }

    const actorInstance = Actor.createActor(idlFactory, { agent, canisterId });
    setActor(actorInstance);
    setIsAuthenticated(true);

    const userPrincipal = await actorInstance.whoami();
    setPrincipal(userPrincipal);

    fetchTasks(actorInstance);
  };

  // Handle login with Internet Identity
  // const login = async () => {
  //   const II_URL = process.env.DFX_NETWORK === "ic"
  //     ? "https://identity.ic0.app"
  //     : "http://127.0.0.1:4943/";

  //   await authClient.login({
  //     identityProvider: II_URL,
  //     onSuccess: () => handleAuthenticated(authClient),
  //   });
  // };
  const login = async () => {
    const internetIdentityCanisterId = 'dqerg-34aaa-aaaaa-qaapq-cai'; // Hardcode for now
    await authClient.login({
      identityProvider: process.env.DFX_NETWORK === 'ic'
        ? 'https://identity.ic0.app'
        : `http://${internetIdentityCanisterId}.localhost:4943/`, // Recommended URL
      onSuccess: () => handleAuthenticated(authClient),
    });
  };

  const logout = async () => {
    await authClient.logout();
    setIsAuthenticated(false);
    setPrincipal(null);
    setTasks([]);
    setActor(null);
  };

  // Fetch user's tasks
  const fetchTasks = async (actorInstance) => {
    if (!actorInstance) return;
    try {
      const taskList = await actorInstance.get_tasks();
      setTasks(taskList);
    } catch (error) {
      console.error("Error fetching tasks:", error);
    }
  };

  // Add a new task
  const handleAddTask = async (event) => {
    event.preventDefault();
    if (!taskName.trim()) return;

    try {
      await actor.add_task(taskName);
      setTaskName("");
      fetchTasks(actor);
    } catch (error) {
      console.error("Error adding task:", error);
    }
  };

  // Delete a task
  const handleDeleteTask = async (taskId) => {
    try {
      await actor.delete_task(BigInt(taskId));
      fetchTasks(actor);
    } catch (error) {
      console.error("Error deleting task:", error);
    }
  };

  return (
    <main style={{ maxWidth: "600px", margin: "20px auto", fontFamily: "Arial, sans-serif" }}>
      <img src="/logo2.svg" alt="DFINITY logo" style={{ display: "block", margin: "0 auto" }} />
      <br />

      {!isAuthenticated ? (
        <div>
          <h2>Welcome to Task Manager</h2>
          <button onClick={login}>Login with Internet Identity</button>
        </div>
      ) : (
        <div>
          <h2>Your Tasks</h2>
          <p>Logged in as: {principal?.toText()}</p>
          <button onClick={logout}>Logout</button>

          <form onSubmit={handleAddTask}>
            <label htmlFor="task">New Task: </label>
            <input
              id="task"
              type="text"
              value={taskName}
              onChange={(e) => setTaskName(e.target.value)}
              placeholder="Enter task name"
              style={{ marginRight: "10px" }}
            />
            <button type="submit">Add Task</button>
          </form>

          <section id="task-list" style={{ marginTop: "20px" }}>
            {tasks.length === 0 ? (
              <p>No tasks yet.</p>
            ) : (
              <ul style={{ listStyle: "none", padding: 0 }}>
                {tasks.map((task) => (
                  <li
                    key={Number(task.id)}
                    style={{
                      display: "flex",
                      justifyContent: "space-between",
                      padding: "10px",
                      borderBottom: "1px solid #ddd",
                    }}
                  >
                    <span>{task.name}</span>
                    <button onClick={() => handleDeleteTask(task.id)}>Delete</button>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </div>
      )}
    </main>
  );
}

export default App;
