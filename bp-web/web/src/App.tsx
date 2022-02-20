import './App.css';
import { Header, System, Security, Configuration } from './modules';

export default function App() {
  return (
    <div className="App">
      <Header />
      <div className="App-body p-3">
        <System />
        <Security />
        <Configuration />
      </div>
    </div>
  );
}
