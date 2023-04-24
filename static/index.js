function join(id=document.getElementById('gameid').value, name=document.getElementById('join_name').value) {
  fetch(`/join/${id}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      name
    })
  }).then(async (res) => {
    const response = await res.text();
    if (res.status !== 200) {
      alert(`Failed to join game: ${response}`);
    } else {
      // Redirects to the game page
      window.location.href = `/${id}/${response}`;
    }
  })
}
function create(settings={players:document.getElementById('players').value}, name=document.getElementById('create_name').value) {
  fetch(`/newgame`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      players: parseInt(settings.players)
    })
  }).then(async (res) => {
    const response = await res.text();
    if (res.status !== 200) {
      alert(`Failed to create game: ${response}`);
    } else {
      // Joins the game after creating it
      join(response, name);
    }
  })
}
if (path_vars) {
  game(path_vars)
}
// Used to update the UI whenever a new event is received
async function updateUI(data) {
  console.log(data)
  document.getElementsByClassName("log")[0].innerText += data.text;
  document.getElementById("events").style.display = data.input ? "block" : "none";
}
// Used to send input
async function sendInput() {
  const input = document.getElementById("message").value;
  const response = await fetch(`/input/${path_vars[0]}/${path_vars[1]}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      input
    })
  });
  if (response.status !== 200) {
    alert(`Input failed due to: ${await response.text()}`);
  }
}
async function game(path_vars) {
  const gameid = path_vars[0];
  const playerid = path_vars[1];
  const response = await fetch(`/events/${gameid}/${playerid}`);
  /*
    Title: How to handle streaming data using fetch?
    Author: Domenic
    Date: May 31 2020
    Availability: https://stackoverflow.com/questions/62121310/how-to-handle-streaming-data-using-fetch
  */
  const reader = response.body.getReader();
  let value, done;
  while (!done) {
    ({ value, done } = await reader.read());
    if (done) {
      return chunks;
    }
    /*
      Title: Uint8Array to string in Javascript
      Author: Vincent Scheib
      Date: Apr 30 2016
      Availability: https://stackoverflow.com/questions/8936984/uint8array-to-string-in-javascript
    */
    console.log(value);
    let data = new TextDecoder().decode(value).split("\n");
    for (let i = 0; i < data.length; i++) {
      console.log(data[i])
      if (data[i]) {
        try {
          updateUI(JSON.parse(data[i]));
        } catch (Error) {
          alert(`Failed to parse: ${data[i]}`);
        }
      }
    }
  }
}