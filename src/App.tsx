import { useState } from "react";
import "./App.css";
import Add from "./Add";

enum Page {
  Videos = "Videos",
  Add = "Add"
}

function App() {
  const [currentPage, setCurrentPage] = useState<Page>(Page.Videos)

  const renderContent = () => {
    switch (currentPage) {
      case Page.Videos:
        return <div>Hi</div>
      case Page.Add:
        return <Add/>
    }
  }

  return (
    <main className="container">
      <div data-tauri-drag-region className="nav-menu">
        <button onClick={() => setCurrentPage(Page.Videos)}>
          <img src="play.svg"/>
        </button>

        <button onClick={() => setCurrentPage(Page.Add)}>
          <img src="add.svg"/>
        </button>
      </div>

      <div className="page">
        <h1>{currentPage}</h1>

        {renderContent()}
      </div>
    </main>
  );
}

export default App;
