// forwardRef: https://github.com/jsx-eslint/eslint-plugin-react/issues/3524
/* eslint-disable react/prop-types */

import { formatTimestamp } from '@/lib/utils';
import React from 'react';

export interface TimeAgoProps extends React.HTMLAttributes<HTMLSpanElement> {
  date: Date;
  justTime?: boolean;
  updateInterval?: number; // in seconds
}

/**
 * @brief A component that displays a timestamp as a human-readable string in real time.
 *
 * @returns "5 minutes ago", "Today at 3:00 PM", "Yesterday at 3:00 PM", "12/31/2020 3:00 PM"
 */
const TimeAgo = React.forwardRef<HTMLSpanElement, TimeAgoProps>(
  ({ date, justTime, updateInterval = 60, ...props }, ref) => {
    const [timestampText, setTimestampText] = React.useState('');

    React.useEffect(() => {
      if (justTime) {
        setTimestampText(formatTimestamp(date, true));
        return;
      }

      // Update the timestamp text immediately when the date prop changes
      setTimestampText(formatTimestamp(date));

      // Make sure we're exactly on the second
      const now = new Date();
      const delay = (updateInterval - now.getSeconds()) * 1000 + (1000 - now.getMilliseconds());

      const timeout = setTimeout(() => {
        setTimestampText(formatTimestamp(date));

        const interval = setInterval(() => {
          setTimestampText(formatTimestamp(date));
        }, updateInterval * 1000);

        return () => clearInterval(interval);
      }, delay);

      return () => clearTimeout(timeout);
    }, [date, justTime]);

    return (
      <span ref={ref} {...props}>
        {timestampText}
      </span>
    );
  },
);
TimeAgo.displayName = 'TimeAgo';

export { TimeAgo };
