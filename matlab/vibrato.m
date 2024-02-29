% Read input WAV file
[x, fs] = audioread('input.wav');
SAMPLERATE = fs; % Use the sample rate from the input file

% Define effect parameters
Width = 0.01; % Base delay width in seconds
Modfreq = 2; % Modulation frequency in Hz

% Ensure parameters are compatible
Delay = Width; % Basic delay of input sample in seconds
DELAY = round(Delay * SAMPLERATE); % Basic delay in number of samples
WIDTH = round(Width * SAMPLERATE); % Modulation width in number of samples
if WIDTH > DELAY
    error('Width greater than basic delay!');
end

MODFREQ = Modfreq / SAMPLERATE; % Modulation frequency in number of samples
LEN = length(x); % Number of samples in the WAV file
L = 2 + DELAY + WIDTH * 2; % Length of the entire delay line
Delayline = zeros(L, 1); % Memory allocation for delay line
y = zeros(size(x)); % Memory allocation for output vector

% Process each sample
for n = 1:(LEN-1)
    M = MODFREQ;
    MOD = sin(M * 2 * pi * n);
    TAP = 1 + DELAY + WIDTH * MOD;
    i = floor(TAP);
    frac = TAP - i;
    Delayline = [x(n); Delayline(1:L-1)]; % Update delay line
    % Linear Interpolation
    y(n, 1) = Delayline(i+1) * frac + Delayline(i) * (1 - frac);
end

% Write output to WAV file
audiowrite('output.wav', y, SAMPLERATE);
