import { ControllerBaseProxy } from 'bizify';

type Data = {
  currentTab: string;
  tabs: { name: string, title: string }[];
};

export class AppCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      currentTab: 'configuration',
      tabs: [
        { name: 'configuration', title: 'Configuration' },
        { name: 'system', title: 'System' },
      ],
    };
  }

  handleTabChange = (name: string) => {
    this.data.currentTab = name;
  };
}
