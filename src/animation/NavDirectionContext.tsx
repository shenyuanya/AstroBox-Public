import { createContext, useContext, useState } from 'react';

type Direction = 'forward' | 'back' | 'none';

const NavContext = createContext<{
  direction: Direction;
  setDirection: (d: Direction) => void;
}>({ direction: 'forward', setDirection: () => {} });

export const useNavDirection = () => useContext(NavContext);

export const NavDirectionProvider = ({ children }: { children: React.ReactNode }) => {
  const [direction, setDirection] = useState<Direction>('forward');
  return (
    <NavContext.Provider value={{ direction, setDirection }}>
      {children}
    </NavContext.Provider>
  );
};