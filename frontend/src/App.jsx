import { useEffect, useState } from 'react';
import axios from 'axios';
import './App.css';

export const LOCAL_PORT = 'http://127.0.0.1:8080';

function App() {
  const [users, setUsers] = useState([]);
  const [newUser, setNewUser] = useState({
    username: '',
    email: '',
    age: '',
  });

  useEffect(() => {
    const getUsers = async () => {
      try {
        const { data } = await axios.get(`${LOCAL_PORT}/hi`);
        setUsers(data);
      } catch (error) {
        console.log(error);
      }
    };
    getUsers();
  }, []);

  const handleInputChange = (event) => {
    const { name, value } = event.target;
    setNewUser({
      ...newUser,
      [name]: value,
    });
  };

  // creating a new user
  const createNewUser = async (e) => {
    e.preventDefault();
    try {
      const check = [...Object.values(newUser)].every((val) => val !== '');
      if (check) {
        const newUserObject = {
          username: newUser.username,
          email: newUser.email,
          age: Number(newUser.age),
        };
        const response = await axios.post(`${LOCAL_PORT}/post`, newUserObject);
        if (response.data) {
          console.log('successful');
        }
      }
    } catch (error) {
      console.log(error);
    }
  };

  return (
    <>
      <div>
        {users?.map((user, index) => {
          const { username, email, age } = user;
          return (
            <div key={index}>
              <span>
                {username},{email}, {age}
              </span>
            </div>
          );
        })}
      </div>
      <div>
        <input
          placeholder="username"
          value={newUser.username}
          name="username"
          onChange={handleInputChange}
        />
        <input
          placeholder="email"
          value={newUser.email}
          name="email"
          onChange={handleInputChange}
        />
        <input
          placeholder="age"
          value={newUser.age}
          name="age"
          onChange={handleInputChange}
        />
        <button onClick={(e) => createNewUser(e)}>Add New User</button>
      </div>
    </>
  );
}

export default App;
