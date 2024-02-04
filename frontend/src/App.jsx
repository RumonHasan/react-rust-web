import { useEffect, useState } from 'react';
import axios from 'axios';
import { v4 as uuidv4 } from 'uuid';
import './App.css';

export const LOCAL_PORT = 'http://127.0.0.1:8080';

function App() {
  const [users, setUsers] = useState([]);
  const [newUser, setNewUser] = useState({
    username: '',
    email: '',
    age: '',
  });
  const [editOn, setEditOn] = useState(false);
  const [editId, setEditId] = useState('');

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
    console.log(users);
  }, []);

  const handleInputChange = (event) => {
    const { name, value } = event.target;
    setNewUser({
      ...newUser,
      [name]: value,
    });
  };

  // creating a new user
  const createNewUser = async () => {
    if (editOn) {
      updateUser();
    } else {
      try {
        const check = [...Object.values(newUser)].every((val) => val !== '');
        if (check) {
          const newUserObject = {
            id: uuidv4(),
            username: newUser.username,
            email: newUser.email,
            age: Number(newUser.age),
          };
          const response = await axios.post(
            `${LOCAL_PORT}/post`,
            newUserObject
          );
          if (response.data) {
            console.log('successful');
          }
        }
      } catch (error) {
        console.log(error);
      }
    }
  };

  const updateEdit = (updateId) => {
    setEditOn(true);
    setEditId(updateId);
    const editable_user = users.find((item) => item.id === updateId);
    if (editable_user) {
      setNewUser({
        username: editable_user.username,
        email: editable_user.email,
        age: editable_user.age,
      });
    }
  };

  // updating the user after setting new id and details
  const updateUser = async () => {
    try {
      const updated_user = {
        id: editId,
        ...newUser,
      };
      const patch_user = await axios.patch(
        `${LOCAL_PORT}/update/${editId}`,
        updated_user
      );
      if (patch_user) {
        setEditOn(false);
        setEditId('');
        setNewUser({
          username: '',
          email: '',
          age: '',
        });
      }
    } catch (error) {
      console.log(error);
    }
  };

  // for deleting user
  const deleteUser = async (e, deleteId) => {
    e.preventDefault();
    try {
      console.log(deleteId);
      const deleted_user = await axios.delete(
        `${LOCAL_PORT}/delete/${deleteId}`
      );
      if (deleted_user) {
        console.log('deleted');
      }
    } catch (error) {
      console.log(error);
    }
  };

  return (
    <>
      <div>
        {users?.map((user, index) => {
          const { id, username, email, age } = user;
          return (
            <div
              key={index}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                gap: '10px',
              }}
            >
              <span>
                {username},{email}, {age}
              </span>
              <div>
                <button onClick={(e) => deleteUser(e, id)}>Delete User</button>
                <button onClick={() => updateEdit(id)}>Edit User</button>
              </div>
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
        <button onClick={() => createNewUser()}>
          {editOn ? 'Edit User' : 'Add New User'}
        </button>
      </div>
    </>
  );
}

export default App;
