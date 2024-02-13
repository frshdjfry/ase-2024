[input, Fs_input] = audioread('input2.wav');
[rustOutput, Fs_rust] = audioread('output2_rust_iir.wav'); 
[matlabOutput, Fs_matlab] = audioread('output2_matlab_iir.wav'); 


assert(Fs_rust == Fs_matlab, 'Sampling rates do not match.');
assert(length(rustOutput) == length(matlabOutput), 'Signal lengths do not match.');


t = (0:length(rustOutput)-1) / Fs_rust; 

figure;


subplot(4,1,1); 
plot(t, input);
title('Input Wave');
xlabel('Time (s)');
ylabel('Amplitude');

subplot(4,1,2); 
plot(t, rustOutput);
title('Rust-processed Output');
xlabel('Time (s)');
ylabel('Amplitude');

subplot(4,1,3); 
plot(t, matlabOutput);
title('MATLAB-processed Output');
xlabel('Time (s)');
ylabel('Amplitude');

subplot(4,1,4); 
plot(t, difference);
title('Difference Between Outputs');
xlabel('Time (s)');
ylabel('Amplitude');

sgtitle('Comparison of Input, Rust and MATLAB Processed Outputs');