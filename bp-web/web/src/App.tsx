import { useController } from 'bizify';
import { TabNav } from './components';
import { Header, Control, Configuration, System } from './modules';
import { AppCtrl } from './AppModel';
import './App.css';

export default function App() {
  const vm = useController<AppCtrl>(AppCtrl);

  return (
    <div className="App">
      <Header />
      <Control />
      <TabNav
        className="m-3"
        current={vm.data.currentTab}
        onChange={vm.handleTabChange}
        items={vm.data.tabs}
      >
        <TabNav.Nav name="configuration" >
          <Configuration />
        </TabNav.Nav>
        <TabNav.Nav name="system">
          <System />
        </TabNav.Nav>
      </TabNav>
    </div>
  );
}
