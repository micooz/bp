import { useController } from 'bizify';
import { Caption, TabNav, TabNavItem } from './components';
import { Header, Control, Footer } from './modules';
import { AppCtrl } from './AppModel';
import './App.css';

export default function App() {
  const vm = useController<AppCtrl>(AppCtrl);
  const { data: vmData } = vm;

  return (
    <div className="App">
      <Header />
      <div className="m-3">
        <Caption>Controller</Caption>
        <Control />

        <Caption>Settings</Caption>
        <TabNav
          style={{ marginBottom: 0 }}
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
      <Footer />
    </div>
  );
}
