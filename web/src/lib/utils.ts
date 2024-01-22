import { formatDistanceToNow, format, isToday, isYesterday, subMinutes, isWithinInterval } from 'date-fns';

import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * @brief Utility function to format a timestamp
 * @param timestamp The timestamp to format
 * @param justTime Whether to just return the time
 * @returns The formatted timestamp
 *          ... "5 minutes ago", "Today at 3:00 PM", "Yesterday at 3:00 PM", "12/31/2020 3:00 PM"
 */
export function formatTimestamp(timestamp: Date, justTime: boolean = false) {
  if (justTime) {
    return format(timestamp, 'hh:mm a');
  }

  const now = new Date();

  switch (true) {
    case isWithinInterval(timestamp, { start: subMinutes(now, 10), end: now }):
      return formatDistanceToNow(timestamp, {
        addSuffix: true,
      });

    case isToday(timestamp):
      return `Today at ${format(timestamp, 'hh:mm a')}`;

    case isYesterday(timestamp):
      return `Yesterday at ${format(timestamp, 'hh:mm a')}`;

    default:
      return format(timestamp, 'MM/dd/yyyy hh:mm a');
  }
}
