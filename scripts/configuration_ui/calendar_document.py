"""Independent RFC 5545 checks for the emitted native calendar profile."""
from datetime import date, datetime
import re

TEXT = {'SUMMARY', 'DESCRIPTION', 'LOCATION', 'CATEGORIES', 'UID', 'X-WR-CALNAME',
        'X-ORG-ID', 'X-ORG-TIMESTAMP', 'X-ORG-DTSTART', 'X-ORG-DUE',
        'X-ORG-RECURRENCE-LIMIT'}


def unescape(value):
    output = []
    at = 0
    while at < len(value):
        char = value[at]
        if char == '\\':
            at += 1
            assert at < len(value) and value[at] in '\\,;nN', value
            char = '\n' if value[at] in 'nN' else value[at]
        output.append(char)
        at += 1
    return ''.join(output)


def one(component, key):
    values = component.get(key, [])
    assert len(values) == 1, (key, values)
    return values[0]


def temporal(component, key):
    plain = component.get(key)
    all_day = component.get(key + ';VALUE=DATE')
    assert not (plain and all_day), key
    if plain:
        value = one(component, key)
        assert re.fullmatch(r'[0-9]{8}T[0-9]{6}Z', value), value
        return datetime.strptime(value, '%Y%m%dT%H%M%SZ')
    if all_day:
        value = one(component, key + ';VALUE=DATE')
        assert re.fullmatch(r'[0-9]{8}', value), value
        return datetime.strptime(value, '%Y%m%d').date()
    return None


class Document:
    def __init__(self, data):
        assert data.endswith(b'\r\n') and not data.startswith(b'\xef\xbb\xbf')
        assert b'\n' not in data.replace(b'\r\n', b'')
        assert b'\r' not in data.replace(b'\r\n', b'')
        lines = []
        for line in data.split(b'\r\n')[:-1]:
            assert 0 < len(line) <= 75, len(line)
            line.decode('utf-8')  # Every physical line ends on a scalar boundary.
            if line.startswith((b' ', b'\t')):
                assert lines
                lines[-1] += line[1:]
            else:
                lines.append(line)
        assert lines[0] == b'BEGIN:VCALENDAR' and lines[-1] == b'END:VCALENDAR'
        assert lines.count(b'BEGIN:VCALENDAR') == 1
        self.components = []
        self.header = {}
        stack = []
        for line in lines:
            key, value = line.decode('utf-8').split(':', 1)
            assert re.fullmatch(r'[A-Z0-9-]+(?:;VALUE=DATE)?', key), key
            assert not any(ord(c) < 32 and c != '\t' for c in value)
            if key == 'BEGIN':
                assert (not stack and value == 'VCALENDAR') or (stack == ['VCALENDAR'] and value in {'VEVENT', 'VTODO'})
                stack.append(value)
                if value != 'VCALENDAR':
                    self.components.append({'KIND': [value]})
            elif key == 'END':
                assert stack and stack.pop() == value
            else:
                assert stack
                target = self.header if stack == ['VCALENDAR'] else self.components[-1]
                target.setdefault(key, []).append(unescape(value) if key in TEXT else value)
        assert not stack and self.components
        assert one(self.header, 'VERSION') == '2.0'
        assert one(self.header, 'CALSCALE') == 'GREGORIAN'
        assert one(self.header, 'PRODID')
        identities = [one(c, 'UID') for c in self.components]
        assert all(identities) and len(set(identities)) == len(identities)
        for component in self.components:
            self.validate(component)

    @staticmethod
    def validate(component):
        one(component, 'SUMMARY')
        assert isinstance(temporal(component, 'DTSTAMP'), datetime)
        start = temporal(component, 'DTSTART')
        end_name = 'DTEND' if one(component, 'KIND') == 'VEVENT' else 'DUE'
        end = temporal(component, end_name)
        if one(component, 'KIND') == 'VEVENT':
            assert start is not None and end is not None
        if start is not None and end is not None:
            assert type(start) is type(end) and end > start, (start, end)
        if 'RRULE' in component:
            assert start is not None
            rule = dict(token.split('=') for token in one(component, 'RRULE').split(';'))
            assert set(rule) == {'FREQ', 'INTERVAL'}
            assert rule['FREQ'] in {'HOURLY', 'DAILY', 'WEEKLY', 'MONTHLY', 'YEARLY'}
            assert 1 <= int(rule['INTERVAL']) <= 2147483647
        if 'STATUS' in component:
            assert one(component, 'STATUS') in {'NEEDS-ACTION', 'COMPLETED'}

    def values(self, name):
        return [value for component in self.components for value in component.get(name, [])]


def fixture(data):
    document = Document(data)
    assert len(document.components) == 6
    entries = {one(c, 'X-ORG-ID'): c for c in document.components}
    task = entries['fixture-task']
    assert one(task, 'SUMMARY') == 'Write'
    assert one(task, 'STATUS') == 'NEEDS-ACTION'
    assert temporal(task, 'DTSTART') == datetime(2024, 3, 4, 9)
    assert temporal(task, 'DUE') == datetime(2024, 3, 4, 17)
    assert one(task, 'LOCATION') == 'Desk, room; north\\side'
    assert one(task, 'CLASS') == 'PRIVATE'
    description = one(task, 'DESCRIPTION')
    for text in ['Body, with; punctuation \\ and Unicode 界.', 'END:VCALENDAR\nBEGIN:VEVENT', 'touch marker\n<2024-01-03 Wed>']:
        assert text in description, (text, description)
    night = entries['fixture-night']
    assert temporal(night, 'DTSTART') == datetime(2024, 3, 4, 23, 30)
    assert temporal(night, 'DTEND') == datetime(2024, 3, 5, 0, 30)
    leap = entries['fixture-leap']
    assert temporal(leap, 'DTSTART') == date(2024, 2, 28)
    assert temporal(leap, 'DTEND') == date(2024, 3, 2)
    assert one(entries['fixture-weekly'], 'RRULE') == 'FREQ=WEEKLY;INTERVAL=2'
    restart = entries['fixture-restart']
    assert 'RRULE' not in restart and 'X-ORG-RECURRENCE-LIMIT' in restart
    assert one(restart, 'X-ORG-TIMESTAMP') == '<2024-03-04 Mon .+1w>'
    assert one(entries['fixture-done'], 'STATUS') == 'COMPLETED'
    assert 'COMPLETED' not in entries['fixture-done']


if __name__ == '__main__':
    import sys
    from pathlib import Path
    data = Path(sys.argv[1]).read_bytes()
    fixture(data) if '--fixture' in sys.argv[2:] else Document(data)
