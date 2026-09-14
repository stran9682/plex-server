import "./App.css";

function App() {
  return (
    <main className="container">
      <div data-tauri-drag-region className="nav-menu">
        <button>
          <img src="home.svg"/>
        </button>

        <button>
          <img src="add.svg"/>
        </button>
      </div>

      <div className="page">
        <h1>Videos</h1>
      </div>
    </main>
  );
}

export default App;
