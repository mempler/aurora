import { Button, TimeAgo } from '@/components/ui';

const App = () => {
  return (
    <div>
      <h1 className="text-3xl font-bold underline">Hello world!</h1>
      <Button>Click me</Button>
      <TimeAgo date={new Date()} />
    </div>
  );
};

export default App;
