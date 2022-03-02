import { useController } from 'bizify';
import { TabNav, TabNavItem } from './components';
import { Header, Control } from './modules';
import { AppCtrl } from './AppModel';
import './App.css';

export default function App() {
  const vm = useController<AppCtrl>(AppCtrl);
  const { data: vmData } = vm;

  return (
    <div className="App">
      <Header />
      <Control />
      <TabNav
        className="m-3"
        current={vmData.currentTab}
        onChange={vm.handleTabChange}
        items={vmData.tabs}
      >
        {vmData.tabs.map(item => (
          <TabNavItem key={item.name} name={item.name} >
            <item.component />
          </TabNavItem>
        ))}
      </TabNav>
    </div>
  );
}
