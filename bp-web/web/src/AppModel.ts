import { ControllerBaseProxy } from 'bizify';
import { Configuration, Logs, System } from './modules';

type Data = {
  currentTab: string;
  tabs: { name: string, title: string; component: React.FC }[];
};

export class AppCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      currentTab: 'configuration',
      tabs: [
        { name: 'configuration', title: 'Configuration', component: Configuration },
        { name: 'log', title: 'Log', component: Logs },
        { name: 'system', title: 'System', component: System },
      ],
    };
  }

  handleTabChange = (name: string) => {
    this.data.currentTab = name;
  };
}
