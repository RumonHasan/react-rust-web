import { useEffect, useState } from 'react';
import axios from 'axios';
import './App.css';

function App() {
  const [users, setUsers] = useState([]);

  useEffect(() => {
    const getUsers = async () => {
      try {
        const { data } = await axios.get('http://127.0.0.1:8080/hi');
        setUsers(data);
        console.log(data);
      } catch (error) {
        console.log(error);
      }
    };

    getUsers();
  }, []);

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
    </>
  );
}

export default App;
