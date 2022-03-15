import { ControllerBaseProxy } from 'bizify';
import { Configuration, Acl, Logs, Monitor } from './modules';

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
        { name: 'acl', title: 'ACL', component: Acl },
        { name: 'log', title: 'Log', component: Logs },
        { name: 'monitor', title: 'Monitor', component: Monitor },
      ],
    };
  }

  handleTabChange = (name: string) => {
    this.data.currentTab = name;
  };
}
